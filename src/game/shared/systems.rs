use super::{
    constants::{C, DAYS_PER_SECOND_UOM, G},
    helpers::{get_translation_from_position, length_to_pixel, sprite_pixel_radius_to_scale},
    types::{Mass, Position, Radius, Velocity},
};
use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};
use bevy::prelude::*;
use glam::DVec2;
use uom::si::{acceleration::meter_per_second_squared, f64::Acceleration as UomAcceleration};

// Camera.

pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(SCREEN_WIDTH_PX as f32 / 2.0, SCREEN_HEIGHT_PX as f32 / 2.0, 0.0);

    commands.spawn(Camera2dBundle { transform, ..Default::default() });
}

// Basic scale / velocity / position.

pub fn scale_update(mut query: Query<(&mut Transform, &Radius)>) {
    for (mut transform, radius) in query.iter_mut() {
        let scale = sprite_pixel_radius_to_scale(length_to_pixel(radius.value));
        transform.scale = scale;
    }
}

pub fn position_update(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in query.iter_mut() {
        let time_elapsed = *DAYS_PER_SECOND_UOM * time.delta_seconds() as f64;

        position.x += velocity.x * time_elapsed;
        position.y += velocity.y * time_elapsed;
    }
}

pub fn translation_update(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = get_translation_from_position(position);
    }
}

// Velocity based on gravitation.

pub fn velocity_update(mut query: Query<(&mut Velocity, Entity, &Position)>, masses: Query<(Entity, &Position, &Mass)>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * time.delta_seconds() as f64;

    for (mut velocity, entity, position) in query.iter_mut() {
        if velocity.x.value == 0.0 || velocity.y.value == 0.0 {
            continue;
        }

        let mut total_gravitational_acceleration_x = UomAcceleration::new::<meter_per_second_squared>(0.0);
        let mut total_gravitational_acceleration_y = UomAcceleration::new::<meter_per_second_squared>(0.0);

        for (other_entity, other_position, other_mass) in masses.iter() {
            if entity == other_entity {
                continue;
            }

            let direction = DVec2::new((other_position.x - position.x).value, (other_position.y - position.y).value);
            let direction = direction.normalize();

            let delta_x = position.x - other_position.x;
            let delta_y = position.y - other_position.y;
            let distance_squared = delta_x * delta_x + delta_y * delta_y;
            let distance = distance_squared.sqrt();

            let gravitational_acceleration = (*G * other_mass.value) / distance_squared;

            let mut relativistic_adjustment = 1.0 - (2.0 * *G * other_mass.value / (*C * *C * distance)).value;

            if relativistic_adjustment <= 0.0 {
                relativistic_adjustment = 0.0;
            }

            let gravitational_acceleration_x = direction.x * gravitational_acceleration * relativistic_adjustment;
            let gravitational_acceleration_y = direction.y * gravitational_acceleration * relativistic_adjustment;

            total_gravitational_acceleration_x += gravitational_acceleration_x;
            total_gravitational_acceleration_y += gravitational_acceleration_y;
        }

        velocity.x += total_gravitational_acceleration_x * time_elapsed;
        velocity.y += total_gravitational_acceleration_y * time_elapsed;

        if velocity.scalar() > *C {
            println!("AHHHHHHHHHHHHHH...fuck.");
        }
    }
}
