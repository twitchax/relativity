use super::{
    constants::{C, DAYS_PER_SECOND_UOM, G},
    helpers::{get_translation_from_position, has_collided, length_to_pixel, planet_sprite_pixel_radius_to_scale, rocket_sprite_pixel_radius_to_scale},
    types::{LaunchState, Mass, PlanetSprite, Position, Radius, RocketSprite, Velocity},
};
use crate::{
    game::{destination::Destination, fade::FadeState, object::Planet, player::shared::Player},
    shared::state::{AppState, GameState},
};
use bevy::prelude::*;
use glam::DVec2;
use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration as UomAcceleration, Length as UomLength, Mass as UomMass},
};

// Escape button.

pub fn exit_level_check(keyboard_input: ResMut<ButtonInput<KeyCode>>, mut fade: ResMut<FadeState>, mut game_state: ResMut<NextState<GameState>>, mut launch_state: ResMut<LaunchState>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Reset game-internal state immediately so UI cleans up.
        game_state.set(GameState::Paused);
        *launch_state = LaunchState::Idle;

        // Fade-out to menu.
        fade.start_fade_out(AppState::Menu, GameState::Paused);
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

// Pure functions.

/// Compute the rotation angle (radians) for a rocket sprite given its velocity components.
///
/// The angle is measured from the positive-y axis (sprite "up") clockwise,
/// returned as a value suitable for `Quat::from_rotation_z`.
#[must_use]
pub(crate) fn calculate_rocket_rotation(vx: f64, vy: f64) -> f32 {
    let velocity = DVec2::new(vx, vy).normalize();
    let rotation = velocity.y.atan2(velocity.x) - std::f64::consts::FRAC_PI_2;

    #[allow(clippy::cast_possible_truncation)]
    let rotation = rotation as f32;
    rotation
}

pub fn rocket_rotation_update(mut query: Query<(&mut Transform, &Velocity), With<RocketSprite>>) {
    for (mut transform, velocity) in &mut query {
        let rotation = calculate_rocket_rotation(velocity.x.value, velocity.y.value);
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}

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

// Pure functions.

/// Compute the gravitational acceleration (x, y) exerted on a body at `pos` by a single mass at `other_pos`.
///
/// Includes a relativistic adjustment factor that reduces acceleration near the Schwarzschild radius.
#[must_use]
pub(crate) fn calculate_gravitational_acceleration(pos_x: UomLength, pos_y: UomLength, other_pos_x: UomLength, other_pos_y: UomLength, other_mass: UomMass) -> (UomAcceleration, UomAcceleration) {
    let direction = DVec2::new((other_pos_x - pos_x).value, (other_pos_y - pos_y).value);
    let direction = direction.normalize();

    let delta_x = pos_x - other_pos_x;
    let delta_y = pos_y - other_pos_y;
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    let distance = distance_squared.sqrt();

    let gravitational_acceleration = (*G * other_mass) / distance_squared;

    let relativistic_adjustment = calculate_relativistic_adjustment(other_mass, distance);

    let accel_x = direction.x * gravitational_acceleration * relativistic_adjustment;
    let accel_y = direction.y * gravitational_acceleration * relativistic_adjustment;

    (accel_x, accel_y)
}

/// Compute the relativistic adjustment factor for gravitational acceleration.
///
/// Returns a value in `[0.0, 1.0]` that reduces acceleration near the Schwarzschild radius.
#[must_use]
pub(crate) fn calculate_relativistic_adjustment(mass: UomMass, distance: UomLength) -> f64 {
    let adjustment = 1.0 - (2.0 * *G * mass / (*C * *C * distance)).value;

    if adjustment <= 0.0 {
        0.0
    } else {
        adjustment
    }
}

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

            let (accel_x, accel_y) = calculate_gravitational_acceleration(position.x, position.y, other_position.x, other_position.y, other_mass.value);

            total_gravitational_acceleration_x += accel_x;
            total_gravitational_acceleration_y += accel_y;
        }

        velocity.x += total_gravitational_acceleration_x * time_elapsed;
        velocity.y += total_gravitational_acceleration_y * time_elapsed;

        if velocity.scalar() > *C {
            // Velocity exceeds speed of light — should not happen in normal gameplay.
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
    }

    for (planet_position, planet_radius) in &planet_query {
        if has_collided((player_position, player_radius), (planet_position, planet_radius)) {
            game_state.set(GameState::Failed);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::{length::kilometer, mass::kilogram};

    fn km(v: f64) -> UomLength {
        UomLength::new::<kilometer>(v)
    }

    fn kg(v: f64) -> UomMass {
        UomMass::new::<kilogram>(v)
    }

    // --- calculate_relativistic_adjustment ---

    #[test]
    fn relativistic_adjustment_far_from_mass_is_near_one() {
        let mass = kg(1.989e30);
        let distance = km(1_000_000_000.0);
        let adj = calculate_relativistic_adjustment(mass, distance);
        assert!(adj > 0.99, "far from mass, adjustment should be near 1.0, got {adj}");
        assert!(adj <= 1.0);
    }

    #[test]
    fn relativistic_adjustment_clamps_at_zero() {
        // Extremely massive body at tiny distance triggers the clamp.
        let mass = kg(1.0e60);
        let distance = km(1.0);
        let adj = calculate_relativistic_adjustment(mass, distance);
        assert_relative_eq!(adj, 0.0);
    }

    #[test]
    fn relativistic_adjustment_is_between_zero_and_one() {
        let mass = kg(1.989e38);
        let distance = km(500_000_000.0);
        let adj = calculate_relativistic_adjustment(mass, distance);
        assert!(adj >= 0.0);
        assert!(adj <= 1.0);
    }

    #[test]
    fn relativistic_adjustment_closer_means_smaller() {
        let mass = kg(1.989e38);
        let adj_far = calculate_relativistic_adjustment(mass, km(1_000_000_000.0));
        let adj_close = calculate_relativistic_adjustment(mass, km(100_000_000.0));
        assert!(adj_far > adj_close, "closer should reduce the adjustment");
    }

    #[test]
    fn relativistic_adjustment_more_mass_means_smaller() {
        let distance = km(500_000_000.0);
        let adj_light = calculate_relativistic_adjustment(kg(1.0e30), distance);
        let adj_heavy = calculate_relativistic_adjustment(kg(1.0e38), distance);
        assert!(adj_light > adj_heavy, "more mass should reduce the adjustment");
    }

    // --- calculate_gravitational_acceleration ---

    #[test]
    fn gravitational_acceleration_points_toward_mass() {
        // Mass to the right of the body.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(0.0), kg(1.989e30));
        assert!(ax.get::<meter_per_second_squared>() > 0.0, "accel should point toward mass (positive x)");
        assert_relative_eq!(ay.get::<meter_per_second_squared>(), 0.0, epsilon = 1e-20);
    }

    #[test]
    fn gravitational_acceleration_points_toward_mass_negative_x() {
        // Mass to the left.
        let (ax, _ay) = calculate_gravitational_acceleration(km(1_000_000.0), km(0.0), km(0.0), km(0.0), kg(1.989e30));
        assert!(ax.get::<meter_per_second_squared>() < 0.0, "accel should point toward mass (negative x)");
    }

    #[test]
    fn gravitational_acceleration_points_toward_mass_y() {
        // Mass above.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(0.0), km(1_000_000.0), kg(1.989e30));
        assert_relative_eq!(ax.get::<meter_per_second_squared>(), 0.0, epsilon = 1e-20);
        assert!(ay.get::<meter_per_second_squared>() > 0.0, "accel should point toward mass (positive y)");
    }

    #[test]
    fn gravitational_acceleration_proportional_to_mass() {
        let (ax_light, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(0.0), kg(1.0e30));
        let (ax_heavy, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(0.0), kg(2.0e30));
        // Heavier mass should produce greater acceleration.
        assert!(ax_heavy.get::<meter_per_second_squared>() > ax_light.get::<meter_per_second_squared>());
    }

    #[test]
    fn gravitational_acceleration_inverse_square_distance() {
        let mass = kg(1.989e30);
        let (ax_close, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(500_000.0), km(0.0), mass);
        let (ax_far, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(0.0), mass);
        // Closer should produce much greater acceleration (roughly 4x for 2x distance).
        assert!(ax_close.get::<meter_per_second_squared>() > ax_far.get::<meter_per_second_squared>());

        let ratio = ax_close.get::<meter_per_second_squared>() / ax_far.get::<meter_per_second_squared>();
        // At far distances where relativistic adjustment ≈ 1, ratio should be close to 4.
        assert!(ratio > 3.5, "inverse-square: ratio should be near 4, got {ratio}");
        assert!(ratio < 4.5, "inverse-square: ratio should be near 4, got {ratio}");
    }

    #[test]
    fn gravitational_acceleration_diagonal_direction() {
        // Mass at (1M, 1M), body at origin — both components should be positive.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(1_000_000.0), kg(1.989e30));
        assert!(ax.get::<meter_per_second_squared>() > 0.0);
        assert!(ay.get::<meter_per_second_squared>() > 0.0);
    }

    #[test]
    fn gravitational_acceleration_both_components_equal_on_diagonal() {
        // Mass at 45° should produce equal x and y acceleration.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(1_000_000.0), kg(1.989e30));
        assert_relative_eq!(ax.get::<meter_per_second_squared>(), ay.get::<meter_per_second_squared>(), epsilon = 1e-25);
    }

    #[test]
    fn gravitational_acceleration_is_positive_magnitude() {
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(1_000_000.0), kg(1.989e30));
        let magnitude = (ax.get::<meter_per_second_squared>().powi(2) + ay.get::<meter_per_second_squared>().powi(2)).sqrt();
        assert!(magnitude > 0.0);
    }

    #[test]
    fn gravitational_acceleration_with_strong_relativistic_clamp() {
        // Extremely massive body at small distance — should not panic, acceleration should be zero due to clamp.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1.0), km(0.0), kg(1.0e60));
        assert_relative_eq!(ax.get::<meter_per_second_squared>(), 0.0);
        assert_relative_eq!(ay.get::<meter_per_second_squared>(), 0.0);
    }

    // --- calculate_rocket_rotation ---

    #[test]
    fn rocket_rotation_moving_up_is_zero() {
        // Moving straight up (+y) → rotation should be 0 (sprite "up" aligned with velocity).
        let rot = calculate_rocket_rotation(0.0, 1.0);
        assert_relative_eq!(rot, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_moving_right() {
        // Moving right (+x) → should rotate -π/2 (90° clockwise).
        let rot = calculate_rocket_rotation(1.0, 0.0);
        assert_relative_eq!(rot, -std::f32::consts::FRAC_PI_2, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_moving_left() {
        // Moving left (-x) → should rotate +π/2 (90° counter-clockwise).
        let rot = calculate_rocket_rotation(-1.0, 0.0);
        assert_relative_eq!(rot, std::f32::consts::FRAC_PI_2, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_moving_down() {
        // Moving down (-y) → should rotate ±π (180°).
        let rot = calculate_rocket_rotation(0.0, -1.0);
        assert_relative_eq!(rot.abs(), std::f32::consts::PI, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_diagonal_up_right() {
        // Moving up-right at 45° → should rotate -π/4.
        let rot = calculate_rocket_rotation(1.0, 1.0);
        assert_relative_eq!(rot, -std::f32::consts::FRAC_PI_4, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_diagonal_up_left() {
        // Moving up-left at 45° → should rotate +π/4.
        let rot = calculate_rocket_rotation(-1.0, 1.0);
        assert_relative_eq!(rot, std::f32::consts::FRAC_PI_4, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_diagonal_down_right() {
        // Moving down-right at 45° → should rotate -3π/4.
        let rot = calculate_rocket_rotation(1.0, -1.0);
        assert_relative_eq!(rot, -3.0 * std::f32::consts::FRAC_PI_4, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_diagonal_down_left() {
        // Moving down-left at 45° → atan2(-1, -1) - π/2 = -3π/4 - π/2 = -5π/4.
        let rot = calculate_rocket_rotation(-1.0, -1.0);
        #[allow(clippy::cast_possible_truncation)]
        let expected = (-std::f64::consts::FRAC_PI_4 * 3.0 - std::f64::consts::FRAC_PI_2) as f32;
        assert_relative_eq!(rot, expected, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_magnitude_does_not_affect_angle() {
        // Scaling velocity should not change the rotation angle.
        let rot_small = calculate_rocket_rotation(1.0, 1.0);
        let rot_large = calculate_rocket_rotation(100.0, 100.0);
        assert_relative_eq!(rot_small, rot_large, epsilon = 1e-6);
    }

    #[test]
    fn rocket_rotation_asymmetric_velocity() {
        // Non-unit, non-diagonal velocity: (3, 4) → angle = atan2(4, 3) - π/2.
        let rot = calculate_rocket_rotation(3.0, 4.0);
        #[allow(clippy::cast_possible_truncation)]
        let expected = (4.0_f64.atan2(3.0) - std::f64::consts::FRAC_PI_2) as f32;
        assert_relative_eq!(rot, expected, epsilon = 1e-6);
    }
}
