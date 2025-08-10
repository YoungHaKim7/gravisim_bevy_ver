use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

mod body;
use body::Body;

const GRAVITY_CONST: f32 = 0.0005;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<Body>()
        .init_resource::<SelectedBodyState>()
        .init_resource::<ElasticCollisionsEnabled>() // Initialize the resource
        .add_systems(Startup, (setup, hud_setup))
        .add_systems(
            Update,
            (
                update_bodies,
                compute_gravity_system,
                elastic_collision_system,
                body_sprite_system,
                camera_control_system,
                hud_update_system,
                editor_input_system,
            ),
        ) // Add elastic_collision_system
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Create a common circle mesh
    let circle_mesh = meshes.add(Circle::new(1.0)); // Unit circle

    // Spawn a few bodies for testing
    commands.spawn((
        Body::new(0.0, 0.0, 0.0, 0.0, 1000.0, 50.0),
        MaterialMesh2dBundle {
            mesh: circle_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(50.0)), // Scale to initial size
            ..default()
        },
    ));

    commands.spawn((
        Body::new(200.0, 0.0, 0.0, 2.0, 1.0, 20.0),
        MaterialMesh2dBundle {
            mesh: circle_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.5, 0.5, 1.0))),
            transform: Transform::from_xyz(200.0, 0.0, 0.0).with_scale(Vec3::splat(20.0)),
            ..default()
        },
    ));

    commands.spawn((
        Body::new(-200.0, 0.0, 0.0, -2.0, 1.0, 20.0),
        MaterialMesh2dBundle {
            mesh: circle_mesh.clone().into(),
            material: materials.add(ColorMaterial::from(Color::rgb(1.0, 0.5, 0.5))),
            transform: Transform::from_xyz(-200.0, 0.0, 0.0).with_scale(Vec3::splat(20.0)),
            ..default()
        },
    ));
}

fn update_bodies(mut query: Query<&mut Body>, time: Res<Time>) {
    let mult = time.delta_seconds() * 400.0; // Equivalent to original time_mult

    for mut body in query.iter_mut() {
        body.past_x = body.x;
        body.past_y = body.y;

        body.x += (body.v_x * mult) + (0.5 * body.a_x * mult.powi(2));
        body.y += (body.v_y * mult) + (0.5 * body.a_y * mult.powi(2));

        body.v_x += (body.a_x + body.past_a_x) * mult * 0.5;
        body.v_y += (body.a_y + body.past_a_y) * mult * 0.5;

        body.past_a_x = body.a_x * mult;
        body.past_a_y = body.a_y * mult;

        // Reset acceleration for next frame
        body.a_x = 0.0;
        body.a_y = 0.0;
    }
}

fn compute_gravity_system(mut query: Query<&mut Body>) {
    let mut bodies = query.iter_mut().collect::<Vec<Mut<Body>>>();
    let num_bodies = bodies.len();

    for i in 0..num_bodies {
        for j in (i + 1)..num_bodies {
            let (mut body1, mut body2) = {
                let (b1, b2) = bodies.split_at_mut(j);
                (b1[i].as_mut(), b2[0].as_mut())
            };

            let min_distance = 0.0001;
            let direction = (body2.x - body1.x, body2.y - body1.y);
            let mut distance = ((body2.x - body1.x).powi(2) + (body2.y - body1.y).powi(2)).sqrt();
            if distance < min_distance {
                distance = min_distance;
            }
            let unit_direction = (direction.0 / distance, direction.1 / distance);
            let force_scalar = GRAVITY_CONST * body1.mass * body2.mass / distance.powi(2);

            // Apply force to body1
            let acc_scalar1 = force_scalar / body1.mass;
            body1.a_x += unit_direction.0 * acc_scalar1;
            body1.a_y += unit_direction.1 * acc_scalar1;

            // Apply opposite force to body2
            let acc_scalar2 = force_scalar / body2.mass;
            body2.a_x -= unit_direction.0 * acc_scalar2;
            body2.a_y -= unit_direction.1 * acc_scalar2;
        }
    }
}

fn body_sprite_system(
    mut query: Query<(&Body, &mut Transform, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (body, mut transform, mut material_handle) in query.iter_mut() {
        transform.translation.x = body.x;
        transform.translation.y = body.y;
        // Set Z to 0 for 2D rendering
        transform.translation.z = 0.0;
        // Update sprite size based on body size
        transform.scale = Vec3::splat(body.size); // Scale the unit circle to the body's size

        // Update color
        if let Some(material) = materials.get_mut(&*material_handle) {
            material.color = body.color;
        }
    }
}

fn camera_control_system(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    time: Res<Time>,
) {
    let mut camera_transform = camera_query.single_mut();
    let mut camera_translation = camera_transform.translation;
    let mut camera_scale = camera_transform.scale.x; // Assuming uniform scale

    let camera_speed = 500.0 * time.delta_seconds();
    let zoom_speed = 0.1f32; // Corrected to f32

    // Keyboard pan
    if keyboard_input.pressed(KeyCode::KeyW) {
        camera_translation.y += camera_speed / camera_scale;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        camera_translation.y -= camera_speed / camera_scale;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        camera_translation.x -= camera_speed / camera_scale;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        camera_translation.x += camera_speed / camera_scale;
    }

    // Mouse wheel zoom
    for event in mouse_wheel_events.read() {
        let scroll_delta = event.y;
        let old_scale = camera_scale;
        camera_scale *= (1.0 + zoom_speed).powf(-scroll_delta);

        // Adjust camera position to zoom towards the center of the screen
        // This is a simplified approach. A more accurate one would involve mouse position.
        camera_translation.x = camera_translation.x * (camera_scale / old_scale);
        camera_translation.y = camera_translation.y * (camera_scale / old_scale);
    }

    camera_transform.translation = camera_translation;
    camera_transform.scale = Vec3::splat(camera_scale);
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct HudControlsText; // New component

fn hud_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/start.ttf"); // Assuming font is in assets/fonts/start.ttf

    commands
        .spawn(
            TextBundle::from_section(
                "FPS: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
        )
        .insert(HudText);

    commands.spawn(
        TextBundle::from_section(
            "R: RESET\nH: TOGGLE HUD\nSCROLL: ZOOM\nZ/X: CHANGE SIZE\nC/V: CHANGE DENSITY\nE: TOGGLE ELASTIC (DISABLED)", // Updated text
            TextStyle {
                font: font.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    ).insert(HudControlsText); // Insert new component
}

fn hud_update_system(
    mut query: Query<(&mut Text, Option<&HudText>, Option<&HudControlsText>)>, // Combined query
    time: Res<Time>,
    elastic_collisions_enabled: Res<ElasticCollisionsEnabled>,
) {
    for (mut text, is_fps_text, is_controls_text) in query.iter_mut() {
        if is_fps_text.is_some() {
            text.sections[0].value = format!("FPS: {:.0}", 1.0 / time.delta_seconds());
        } else if is_controls_text.is_some() {
            let controls_text = format!(
                "R: RESET\nH: TOGGLE HUD\nSCROLL: ZOOM\nZ/X: CHANGE SIZE\nC/V: CHANGE DENSITY\nE: TOGGLE ELASTIC ({})",
                if elastic_collisions_enabled.0 { "ENABLED" } else { "DISABLED" }
            );
            text.sections[0].value = controls_text;
        }
    }
}

#[derive(Resource, Default)]
struct SelectedBodyState {
    pos_selected: bool,
    selected_pos: Vec2,
    selected_vel: Vec2,
    selected_size: f32,
    selected_density: f32,
}

#[derive(Resource, Default)]
struct ElasticCollisionsEnabled(bool);

fn elastic_collision_system(
    mut query: Query<&mut Body>,
    elastic_collisions_enabled: Res<ElasticCollisionsEnabled>,
) {
    if !elastic_collisions_enabled.0 {
        return;
    }

    let mut bodies = query.iter_mut().collect::<Vec<Mut<Body>>>();
    let num_bodies = bodies.len();

    for i in 0..num_bodies {
        for j in (i + 1)..num_bodies {
            let (mut body1, mut body2) = {
                let (b1, b2) = bodies.split_at_mut(j);
                (b1[i].as_mut(), b2[0].as_mut())
            };

            let distance_vec = Vec2::new(body2.x - body1.x, body2.y - body1.y);
            let distance = distance_vec.length();
            let min_distance = body1.size + body2.size;

            if distance < min_distance {
                // Collision detected
                let normal = distance_vec.normalize();
                let relative_velocity = Vec2::new(body1.v_x - body2.v_x, body1.v_y - body2.v_y);
                let impulse_magnitude =
                    2.0 * relative_velocity.dot(normal) / (body1.mass + body2.mass);

                body1.v_x -= impulse_magnitude * body2.mass * normal.x;
                body1.v_y -= impulse_magnitude * body2.mass * normal.y;
                body2.v_x += impulse_magnitude * body1.mass * normal.x;
                body2.v_y += impulse_magnitude * body1.mass * normal.y;

                // Separate bodies to prevent sticking
                let overlap = min_distance - distance;
                let separation_vector = normal * overlap * 0.5;
                body1.x -= separation_vector.x;
                body1.y -= separation_vector.y;
                body2.x += separation_vector.x;
                body2.y += separation_vector.y;
            }
        }
    }
}

fn editor_input_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_body_state: ResMut<SelectedBodyState>,
    mut body_query: Query<Entity, With<Body>>,
    mut camera_transform_query: Query<&mut Transform, With<Camera2d>>,
    mut elastic_collisions_enabled: ResMut<ElasticCollisionsEnabled>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // Convert mouse to world coordinates
    let mouse_world_pos = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world(camera_transform, cursor)
            .map(|ray| ray.origin.truncate())
    });

    // Debug logging
    if let Some(pos) = mouse_world_pos {
        info!("Mouse world position: {:?}", pos);
    } else {
        info!("Mouse outside window or no position.");
    }

    // Reset simulation
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for entity in body_query.iter() {
            commands.entity(entity).despawn();
        }
        let mut cam_transform = camera_transform_query.single_mut();
        cam_transform.translation = Vec3::ZERO;
        cam_transform.scale = Vec3::ONE;
        selected_body_state.pos_selected = false;
        selected_body_state.selected_size = 50.0;
        selected_body_state.selected_density = 1.0;
    }

    // Toggle elastic collisions
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        elastic_collisions_enabled.0 = !elastic_collisions_enabled.0;
    }

    // Record start position when mouse is pressed
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = mouse_world_pos {
            selected_body_state.pos_selected = true;
            selected_body_state.selected_pos = pos;
            info!("Start pos: {:?}", pos);
        }
    }

    // On release — spawn the body
    if mouse_button_input.just_released(MouseButton::Left) {
        if let Some(end_pos) = mouse_world_pos {
            if selected_body_state.pos_selected {
                let velocity = (end_pos - selected_body_state.selected_pos) / 50.0;
                info!("End pos: {:?}, Velocity: {:?}", end_pos, velocity);

                commands.spawn((
                    Body::new(
                        selected_body_state.selected_pos.x,
                        selected_body_state.selected_pos.y,
                        velocity.x,
                        velocity.y,
                        selected_body_state.selected_density,
                        selected_body_state.selected_size,
                    ),
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::new(1.0)).into(),
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        transform: Transform::from_xyz(
                            selected_body_state.selected_pos.x,
                            selected_body_state.selected_pos.y,
                            0.0,
                        )
                        .with_scale(Vec3::splat(selected_body_state.selected_size)),
                        ..default()
                    },
                ));

                selected_body_state.pos_selected = false;
            }
        }
    }

    // Change size
    let size_speed = 0.2;
    if keyboard_input.pressed(KeyCode::KeyZ) {
        selected_body_state.selected_size += size_speed;
        if selected_body_state.selected_size < 1.0 {
            selected_body_state.selected_size = 1.0;
        }
    }
    if keyboard_input.pressed(KeyCode::KeyX) {
        selected_body_state.selected_size -= size_speed;
        if selected_body_state.selected_size < 1.0 {
            selected_body_state.selected_size = 1.0;
        }
    }

    // Change density
    let density_speed = 0.1;
    if keyboard_input.pressed(KeyCode::KeyC) {
        selected_body_state.selected_density -= density_speed;
        if selected_body_state.selected_density < 1.0 {
            selected_body_state.selected_density = 1.0;
        }
    }
    if keyboard_input.pressed(KeyCode::KeyV) {
        selected_body_state.selected_density += density_speed;
        if selected_body_state.selected_density < 1.0 {
            selected_body_state.selected_density = 1.0;
        }
    }
}

// fn editor_input_system( mut commands: Commands,
//     windows: Query<&Window>,
//     camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
//     mouse_button_input: Res<ButtonInput<MouseButton>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut selected_body_state: ResMut<SelectedBodyState>,
//     mut body_query: Query<Entity, With<Body>>,
//     mut camera_transform_query: Query<&mut Transform, With<Camera2d>>,
//     mut elastic_collisions_enabled: ResMut<ElasticCollisionsEnabled>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let window = windows.single();
//     let (camera, camera_transform) = camera_query.single();

//     // Get mouse position in world coordinates
//     let mouse_world_pos = window.cursor_position().and_then(|cursor| {
//         camera
//             .viewport_to_world(camera_transform, cursor)
//             .map(|ray| ray.origin.truncate())
//     });

//     // Reset simulation
//     if keyboard_input.just_pressed(KeyCode::KeyR) {
//         // Despawn all Body entities
//         for entity in body_query.iter() {
//             commands.entity(entity).despawn();
//         }
//         // Reset camera to default
//         let mut cam_transform = camera_transform_query.single_mut();
//         cam_transform.translation = Vec3::ZERO;
//         cam_transform.scale = Vec3::ONE;
//         selected_body_state.pos_selected = false;
//         selected_body_state.selected_size = 50.0;
//         selected_body_state.selected_density = 1.0;
//     }

//     // Toggle elastic collisions
//     if keyboard_input.just_pressed(KeyCode::KeyE) {
//         elastic_collisions_enabled.0 = !elastic_collisions_enabled.0;
//     }

//     // Body creation
//     //
//     if mouse_button_input.just_pressed(MouseButton::Left) {
//         if let Some(pos) = mouse_world_pos {
//             commands.spawn((
//                 Body::new(
//                     pos.x,
//                     pos.y,
//                     0.0,
//                     0.0,
//                     selected_body_state.selected_density,
//                     selected_body_state.selected_size,
//                 ),
//                 MaterialMesh2dBundle {
//                     mesh: meshes.add(Circle::new(1.0)).into(),
//                     material: materials.add(ColorMaterial::from(Color::WHITE)),
//                     transform: Transform::from_xyz(pos.x, pos.y, 0.0)
//                         .with_scale(Vec3::splat(selected_body_state.selected_size)),
//                     ..default()
//                 },
//             ));
//         }
//     }
//     // if mouse_button_input.just_pressed(MouseButton::Left) {
//     //     if let Some(pos) = mouse_world_pos {
//     //         if !selected_body_state.pos_selected {
//     //             selected_body_state.pos_selected = true;
//     //             selected_body_state.selected_pos = pos;
//     //         }
//     //     }
//     // }

//     // if mouse_button_input.just_released(MouseButton::Left) {
//     //     if selected_body_state.pos_selected {
//     //         if let Some(pos) = mouse_world_pos {
//     //             selected_body_state.pos_selected = false;
//     //             selected_body_state.selected_vel = (pos - selected_body_state.selected_pos) / 50.0;

//     //             commands.spawn((
//     //                 Body::new(
//     //                     selected_body_state.selected_pos.x,
//     //                     selected_body_state.selected_pos.y,
//     //                     selected_body_state.selected_vel.x,
//     //                     selected_body_state.selected_vel.y,
//     //                     selected_body_state.selected_density,
//     //                     selected_body_state.selected_size,
//     //                 ),
//     //                 MaterialMesh2dBundle {
//     //                     mesh: meshes.add(Circle::new(1.0)).into(), // Use Circle mesh
//     //                     material: materials.add(ColorMaterial::from(Color::rgb(1.0, 1.0, 1.0))), // Default color
//     //                     transform: Transform::from_xyz(selected_body_state.selected_pos.x, selected_body_state.selected_pos.y, 0.0).with_scale(Vec3::splat(selected_body_state.selected_size)),
//     //                     ..default()
//     //                 },
//     //             ));
//     //         }
//     //     }
//     // }

//     // Change size
//     let size_speed = 0.2;
//     if keyboard_input.pressed(KeyCode::KeyZ) {
//         selected_body_state.selected_size += size_speed;
//         if selected_body_state.selected_size < 1.0 {
//             selected_body_state.selected_size = 1.0;
//         }
//     }
//     if keyboard_input.pressed(KeyCode::KeyX) {
//         selected_body_state.selected_size -= size_speed;
//         if selected_body_state.selected_size < 1.0 {
//             selected_body_state.selected_size = 1.0;
//         }
//     }

//     // Change density
//     let density_speed = 0.1;
//     if keyboard_input.pressed(KeyCode::KeyC) {
//         selected_body_state.selected_density -= density_speed;
//         if selected_body_state.selected_density < 1.0 {
//             selected_body_state.selected_density = 1.0;
//         }
//     }
//     if keyboard_input.pressed(KeyCode::KeyV) {
//         selected_body_state.selected_density += density_speed;
//         if selected_body_state.selected_density < 1.0 {
//             selected_body_state.selected_density = 1.0;
//         }
//     }
// }
