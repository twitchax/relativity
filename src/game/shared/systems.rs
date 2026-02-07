use crate::{
    game::{destination::Destination, object::Planet, player::shared::Player},
    shared::state::{AppState, GameState},
};

use super::{
    constants::{C, DAYS_PER_SECOND_UOM, G},
    helpers::{get_translation_from_position, has_collided, length_to_pixel, planet_sprite_pixel_radius_to_scale, rocket_sprite_pixel_radius_to_scale},
    types::{Mass, PlanetSprite, Position, Radius, RocketSprite, Velocity},
};
use bevy::prelude::*;
use glam::DVec2;
use uom::si::{acceleration::meter_per_second_squared, f64::Acceleration as UomAcceleration};

// Escape button.

#[allow(clippy::needless_pass_by_value)]
pub fn exit_level_check(keyboard_input: ResMut<ButtonInput<KeyCode>>, mut app_state: ResMut<NextState<AppState>>, mut game_state: ResMut<NextState<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_state.set(AppState::Menu);
        game_state.set(GameState::Paused);
    }
}

// Basic scale / velocity / position.

pub fn planet_scale_update(mut query: Query<(&mut Transform, &Radius), With<PlanetSprite>>) {
    for (mut transform, radius) in &mut query {
        let scale = planet_sprite_pixel_radius_to_scale(length_to_pixel(radius.value));
        transform.scale = scale;
    }
}

pub fn rocket_scale_update(mut query: Query<(&mut Transform, &Radius), With<RocketSprite>>) {
    for (mut transform, radius) in &mut query {
        let scale = rocket_sprite_pixel_radius_to_scale(length_to_pixel(radius.value));
        transform.scale = scale;
    }
}

pub fn rocket_rotation_update(mut query: Query<(&mut Transform, &Velocity), With<RocketSprite>>) {
    for (mut transform, velocity) in &mut query {
        let velocity = DVec2::new(velocity.x.value, velocity.y.value);
        let velocity = velocity.normalize();

        let rotation = velocity.y.atan2(velocity.x) - std::f64::consts::FRAC_PI_2;

        #[allow(clippy::cast_possible_truncation)]
        let rotation = rotation as f32;
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn position_update(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in &mut query {
        let time_elapsed = *DAYS_PER_SECOND_UOM * f64::from(time.delta_secs());

        position.x += velocity.x * time_elapsed;
        position.y += velocity.y * time_elapsed;
    }
}

pub fn translation_update(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut query {
        transform.translation = get_translation_from_position(position);
    }
}

// Velocity based on gravitation.

#[allow(clippy::needless_pass_by_value)]
pub fn velocity_update(mut query: Query<(&mut Velocity, Entity, &Position)>, masses: Query<(Entity, &Position, &Mass)>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * f64::from(time.delta_secs());

    for (mut velocity, entity, position) in &mut query {
        if velocity.x.value == 0.0 || velocity.y.value == 0.0 {
            continue;
        }

        let mut total_gravitational_acceleration_x = UomAcceleration::new::<meter_per_second_squared>(0.0);
        let mut total_gravitational_acceleration_y = UomAcceleration::new::<meter_per_second_squared>(0.0);

        for (other_entity, other_position, other_mass) in &masses {
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

// Collisions.

pub fn collision_check(
    player_query: Query<(&Position, &Radius), With<Player>>,
    planet_query: Query<(&Position, &Radius), With<Planet>>,
    destination_query: Query<(&Position, &Radius), With<Destination>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_position, player_radius)) = player_query.single() else { return };
    let Ok((destination_position, destination_radius)) = destination_query.single() else { return };

    if has_collided((player_position, player_radius), (destination_position, destination_radius)) {
        game_state.set(GameState::Finished);
        println!("success!");
    }

    for (planet_position, planet_radius) in &planet_query {
        if has_collided((player_position, player_radius), (planet_position, planet_radius)) {
            game_state.set(GameState::Paused);
            println!("failed!");
        }
    }
}
