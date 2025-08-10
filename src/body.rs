use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Body {
    pub past_a_x: f32,
    pub past_a_y: f32,
    pub past_x: f32,
    pub past_y: f32,
    pub x: f32,
    pub y: f32,
    pub a_x: f32,
    pub a_y: f32,
    pub v_x: f32,
    pub v_y: f32,
    pub mass: f32,
    pub size: f32,
    pub density: f32,
    pub color: Color,
}

impl Body {
    pub fn new(x: f32, y: f32, v_x: f32, v_y: f32, density: f32, size: f32) -> Self {
        const PI: f32 = std::f32::consts::PI;
        Body {
            past_a_x: 0f32,
            past_a_y: 0f32,
            past_x: 0f32,
            past_y: 0f32,
            x,
            y,
            v_x,
            v_y,
            a_x: 0f32,
            a_y: 0f32,
            mass: (4.0 / 3.0) * PI * size.powi(3) * density,
            size,
            density,
            color: Color::rgb(1.0, 1.0, 1.0),
        }
    }
}
