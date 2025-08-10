use bevy::prelude::*;

mod body;
use body::Body;

const GRAVITY_CONST: f32 = 0.0005;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<Body>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_bodies, compute_gravity_system, body_sprite_system))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn a few bodies for testing
    commands.spawn((
        Body::new(0.0, 0.0, 0.0, 0.0, 1000.0, 50.0),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 1.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));

    commands.spawn(( 
        Body::new(200.0, 0.0, 0.0, 2.0, 1.0, 20.0),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        Body::new(-200.0, 0.0, 0.0, -2.0, 1.0, 20.0),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.5, 0.5),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, 0.0, 0.0),
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

fn body_sprite_system(mut query: Query<(&Body, &mut Transform)>) {
    for (body, mut transform) in query.iter_mut() {
        transform.translation.x = body.x;
        transform.translation.y = body.y;
        // Set Z to 0 for 2D rendering
        transform.translation.z = 0.0;
        // Update sprite size based on body size
        transform.scale = Vec3::new(body.size / 50.0, body.size / 50.0, 1.0);
    }
}