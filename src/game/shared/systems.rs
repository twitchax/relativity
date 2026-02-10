use super::{
    constants::{C, DAYS_PER_SECOND_UOM, G, SOFTENING_LENGTH},
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

// Simulation pause toggle (Space).

pub fn sim_pause_toggle(keyboard_input: Res<ButtonInput<KeyCode>>, current_state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Running => next_state.set(GameState::SimPaused),
            GameState::SimPaused => next_state.set(GameState::Running),
            _ => {}
        }
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

/// Compute the gravitational acceleration (x, y) exerted on a body at `pos` by
/// a single mass at `other_pos`, using Plummer gravitational softening.
///
/// The softened acceleration is:
///
///   a = G·M / (r² + ε²)
///
/// where ε = `SOFTENING_LENGTH`.  At distances r >> ε this is pure Newtonian
/// inverse-square gravity.  At r → 0 it caps at G·M / ε² instead of diverging,
/// providing numerical stability without the physically incorrect behaviour of
/// zeroing out gravity near massive bodies.
#[must_use]
pub(crate) fn calculate_gravitational_acceleration(pos_x: UomLength, pos_y: UomLength, other_pos_x: UomLength, other_pos_y: UomLength, other_mass: UomMass) -> (UomAcceleration, UomAcceleration) {
    let direction = DVec2::new((other_pos_x - pos_x).value, (other_pos_y - pos_y).value);
    let direction = direction.normalize();

    let delta_x = pos_x - other_pos_x;
    let delta_y = pos_y - other_pos_y;
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    let softened_denominator = distance_squared + *SOFTENING_LENGTH * *SOFTENING_LENGTH;

    let gravitational_acceleration = (*G * other_mass) / softened_denominator;

    let accel_x = direction.x * gravitational_acceleration;
    let accel_y = direction.y * gravitational_acceleration;

    (accel_x, accel_y)
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

        // Clamp velocity to just below the speed of light to prevent NaN in
        // Lorentz gamma calculations and preserve relativistic consistency.
        let speed = velocity.scalar();
        if speed > *C {
            let scale = (*C * 0.9999 / speed).value;
            velocity.x = scale * velocity.x;
            velocity.y = scale * velocity.y;
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

    // --- calculate_gravitational_acceleration (with softening) ---

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
        // Use distances well beyond softening length (10M km) so softening is negligible.
        let (ax_close, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(100_000_000.0), km(0.0), mass);
        let (ax_far, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(200_000_000.0), km(0.0), mass);
        // Closer should produce much greater acceleration (roughly 4x for 2x distance).
        assert!(ax_close.get::<meter_per_second_squared>() > ax_far.get::<meter_per_second_squared>());

        let ratio = ax_close.get::<meter_per_second_squared>() / ax_far.get::<meter_per_second_squared>();
        // At far distances (well beyond softening length), ratio should be close to 4.
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
    fn gravitational_acceleration_with_extreme_mass_at_tiny_distance_is_finite() {
        // Extremely massive body at small distance — softening prevents singularity.
        // Acceleration should be finite and positive (pointing toward mass), not zero.
        let (ax, ay) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1.0), km(0.0), kg(1.0e60));
        assert!(ax.get::<meter_per_second_squared>().is_finite(), "acceleration should be finite");
        assert!(ax.get::<meter_per_second_squared>() > 0.0, "acceleration should still point toward mass");
        assert_relative_eq!(ay.get::<meter_per_second_squared>(), 0.0, epsilon = 1e-20);
    }

    // --- gravitational softening ---

    #[test]
    fn softening_caps_acceleration_at_zero_distance() {
        // At r=0 (body on top of mass), acceleration should be capped at G·M/ε²,
        // not diverge to infinity.  Direction is undefined (NaN), so we test that
        // the function doesn't panic and the magnitude is finite.
        // We test near-zero instead of exactly zero to avoid normalize-of-zero.
        let mass = kg(1.989e38);
        let (ax, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(0.01), km(0.0), mass);
        assert!(ax.get::<meter_per_second_squared>().is_finite(), "should be finite at near-zero distance");
        assert!(ax.get::<meter_per_second_squared>() > 0.0, "should still attract toward mass");
    }

    #[test]
    fn softening_has_negligible_effect_at_large_distance() {
        // At distances far beyond the softening length (10M km), the softened
        // acceleration should be very close to pure Newtonian: G·M / r².
        let mass = kg(1.989e30);
        let distance = km(500_000_000.0); // 500M km >> 10M km softening length

        let (ax, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), distance, km(0.0), mass);

        // Pure Newtonian: G·M / r²
        let expected = (*G * mass / (distance * distance)).get::<meter_per_second_squared>();
        let actual = ax.get::<meter_per_second_squared>();

        // Softening should change the result by less than 0.1%.
        assert_relative_eq!(actual, expected, max_relative = 1e-3);
    }

    #[test]
    fn softening_makes_closer_stronger_monotonically() {
        // Even with softening, closer should always mean stronger acceleration.
        let mass = kg(1.989e38);
        let (ax_close, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(100_000.0), km(0.0), mass);
        let (ax_far, _) = calculate_gravitational_acceleration(km(0.0), km(0.0), km(1_000_000.0), km(0.0), mass);

        assert!(
            ax_close.get::<meter_per_second_squared>() > ax_far.get::<meter_per_second_squared>(),
            "closer should produce greater acceleration even with softening"
        );
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
