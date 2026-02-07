use super::shared::Player;
use crate::game::shared::{
    constants::{C, DAYS_PER_SECOND_UOM, G},
    types::{Clock, GameItem, GravitationalGamma, Mass, Position, Velocity, VelocityGamma},
};
use bevy::prelude::*;
use uom::si::f64::{Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity};

// Components / bundles.

#[derive(Bundle, Default)]
pub struct PlayerClockBundle {
    pub item: GameItem,
    pub player: Player,
    pub velocity_gamma: VelocityGamma,
    pub gravitational_gamma: GravitationalGamma,
    pub clock: Clock,
    pub clock_text: Text,
    pub node: Node,
}

// Pure functions.

/// Compute velocity-dependent Lorentz gamma factor: γ = 1 / √(1 - v²/c²).
#[must_use]
pub(crate) fn calculate_velocity_gamma(vx: UomVelocity, vy: UomVelocity, c: UomVelocity) -> f64 {
    let v_squared_div_c_squared = (vx.value * vx.value + vy.value * vy.value) / (c * c);
    1.0 / (1.0 - v_squared_div_c_squared.value).sqrt()
}

/// Compute combined gravitational gamma from all nearby masses.
/// Each mass contributes `γ_g = 1 / √(1 - 2GM/(c²r))`, and the total is the product.
#[must_use]
pub(crate) fn calculate_gravitational_gamma(player_x: UomLength, player_y: UomLength, masses: &[(UomLength, UomLength, UomMass)]) -> f64 {
    let mut total = 1.0f64;

    for &(other_x, other_y, other_mass) in masses {
        let delta_x = player_x - other_x;
        let delta_y = player_y - other_y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let distance = distance_squared.sqrt();

        let mut gravitational_factor = 1.0 - (2.0 * *G * other_mass / (*C * *C * distance)).value;

        if gravitational_factor <= 0.0001 {
            gravitational_factor = 0.0001;
        }

        total *= 1.0 / gravitational_factor.sqrt();
    }

    total
}

/// Advance the player clock by dt, slowed by velocity and gravitational gamma.
#[must_use]
pub(crate) fn calculate_player_clock(dt: UomTime, velocity_gamma: f64, gravitational_gamma: f64, previous_clock: UomTime) -> UomTime {
    previous_clock + dt / velocity_gamma / gravitational_gamma
}

// Startup systems.

pub fn spawn_player_clock(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let clock_text = Text::new("t_p = 00.00 γ_v = 1.00 γ_g = 1.00 v_p = 00.00");

    let node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        left: Val::Px(10.0),
        ..Default::default()
    };

    commands.spawn((
        PlayerClockBundle { clock_text, node, ..Default::default() },
        TextFont {
            font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
            font_size: 40.0,
            ..Default::default()
        },
    ));
}

// Systems.

#[allow(clippy::needless_pass_by_value)]
pub fn player_clock_update(
    mut query: Query<(&mut Clock, &mut VelocityGamma, &mut GravitationalGamma), With<Player>>,
    player_query: Query<(Entity, &Position, &Velocity), With<Player>>,
    masses: Query<(Entity, &Position, &Mass)>,
    time: Res<Time>,
) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * f64::from(time.delta_secs());

    let Ok((mut clock, mut velocity_gamma, mut gravitational_gamma)) = query.single_mut() else {
        return;
    };
    let Ok((player_entity, player_position, player_velocity)) = player_query.single() else { return };

    velocity_gamma.value = calculate_velocity_gamma(player_velocity.x, player_velocity.y, *C);

    let other_masses: Vec<_> = masses.iter().filter(|(e, _, _)| *e != player_entity).map(|(_, pos, mass)| (pos.x, pos.y, mass.value)).collect();
    gravitational_gamma.value = calculate_gravitational_gamma(player_position.x, player_position.y, &other_masses);

    clock.value = calculate_player_clock(time_elapsed, velocity_gamma.value, gravitational_gamma.value, clock.value);
}

pub fn player_clock_text_update(mut query: Query<(&mut Text, &Clock, &VelocityGamma, &GravitationalGamma), With<Player>>) {
    let Ok((mut text, clock, velocity_gamma, gravitational_gamma)) = query.single_mut() else { return };

    let days = clock.value.value / 24.0 / 3600.0;

    // In Bevy 0.17, Text implements Deref<Target = String>, so we use **text to mutate the underlying String.
    **text = format!("t_p = {:2.2} γ_v = {:2.2} γ_g = {:2.2}", days, velocity_gamma.value, gravitational_gamma.value);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::{length::kilometer, mass::kilogram, time::second, velocity::kilometer_per_second};

    fn c() -> UomVelocity {
        *C
    }

    fn make_vel(kms: f64) -> UomVelocity {
        UomVelocity::new::<kilometer_per_second>(kms)
    }

    // --- calculate_velocity_gamma ---

    #[test]
    fn velocity_gamma_at_rest_is_one() {
        let gamma = calculate_velocity_gamma(make_vel(0.0), make_vel(0.0), c());
        assert_relative_eq!(gamma, 1.0);
    }

    #[test]
    fn velocity_gamma_is_at_least_one_low_speed() {
        let gamma = calculate_velocity_gamma(make_vel(100.0), make_vel(0.0), c());
        assert!(gamma >= 1.0);
    }

    #[test]
    fn velocity_gamma_is_at_least_one_diagonal() {
        let gamma = calculate_velocity_gamma(make_vel(1000.0), make_vel(1000.0), c());
        assert!(gamma >= 1.0);
    }

    #[test]
    fn velocity_gamma_increases_with_velocity() {
        let gamma_slow = calculate_velocity_gamma(make_vel(10_000.0), make_vel(0.0), c());
        let gamma_fast = calculate_velocity_gamma(make_vel(200_000.0), make_vel(0.0), c());
        assert!(gamma_fast > gamma_slow);
    }

    #[test]
    fn velocity_gamma_near_c_is_large() {
        // 99% of c ≈ 296_794 km/s
        let gamma = calculate_velocity_gamma(make_vel(0.99 * 299_792.0), make_vel(0.0), c());
        assert!(gamma > 7.0);
    }

    #[test]
    fn velocity_gamma_symmetric_in_x_and_y() {
        let gamma_x = calculate_velocity_gamma(make_vel(100_000.0), make_vel(0.0), c());
        let gamma_y = calculate_velocity_gamma(make_vel(0.0), make_vel(100_000.0), c());
        assert_relative_eq!(gamma_x, gamma_y);
    }

    #[test]
    fn velocity_gamma_negative_velocity_same_as_positive() {
        let gamma_pos = calculate_velocity_gamma(make_vel(100_000.0), make_vel(50_000.0), c());
        let gamma_neg = calculate_velocity_gamma(make_vel(-100_000.0), make_vel(-50_000.0), c());
        assert_relative_eq!(gamma_pos, gamma_neg);
    }

    // --- calculate_gravitational_gamma ---

    #[test]
    fn gravitational_gamma_no_masses_is_one() {
        let gamma = calculate_gravitational_gamma(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), &[]);
        assert_relative_eq!(gamma, 1.0);
    }

    #[test]
    fn gravitational_gamma_is_at_least_one() {
        let gamma = calculate_gravitational_gamma(
            UomLength::new::<kilometer>(1_000_000_000.0),
            UomLength::new::<kilometer>(0.0),
            &[(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), UomMass::new::<kilogram>(1.989e30))],
        );
        assert!(gamma >= 1.0);
    }

    #[test]
    fn gravitational_gamma_closer_means_larger() {
        let mass = UomMass::new::<kilogram>(1.989e38);
        let mass_pos = (UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), mass);

        let gamma_far = calculate_gravitational_gamma(UomLength::new::<kilometer>(1_000_000_000.0), UomLength::new::<kilometer>(0.0), &[mass_pos]);
        let gamma_close = calculate_gravitational_gamma(UomLength::new::<kilometer>(100_000_000.0), UomLength::new::<kilometer>(0.0), &[mass_pos]);
        assert!(gamma_close > gamma_far);
    }

    #[test]
    fn gravitational_gamma_more_mass_means_larger() {
        let pos = UomLength::new::<kilometer>(500_000_000.0);

        let gamma_light = calculate_gravitational_gamma(
            pos,
            UomLength::new::<kilometer>(0.0),
            &[(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), UomMass::new::<kilogram>(1.0e30))],
        );
        let gamma_heavy = calculate_gravitational_gamma(
            pos,
            UomLength::new::<kilometer>(0.0),
            &[(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), UomMass::new::<kilogram>(1.0e38))],
        );
        assert!(gamma_heavy > gamma_light);
    }

    #[test]
    fn gravitational_gamma_multiple_masses_compound() {
        let mass = UomMass::new::<kilogram>(1.989e38);
        let mass_entry = (UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), mass);

        let gamma_one = calculate_gravitational_gamma(UomLength::new::<kilometer>(500_000_000.0), UomLength::new::<kilometer>(0.0), &[mass_entry]);
        let gamma_two = calculate_gravitational_gamma(
            UomLength::new::<kilometer>(500_000_000.0),
            UomLength::new::<kilometer>(0.0),
            &[mass_entry, (UomLength::new::<kilometer>(1_000_000_000.0), UomLength::new::<kilometer>(0.0), mass)],
        );
        assert!(gamma_two > gamma_one);
    }

    #[test]
    fn gravitational_gamma_clamps_factor_above_zero() {
        // Place very close to an extremely massive body to trigger the 0.0001 clamp.
        let gamma = calculate_gravitational_gamma(
            UomLength::new::<kilometer>(1.0),
            UomLength::new::<kilometer>(0.0),
            &[(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), UomMass::new::<kilogram>(1.0e60))],
        );
        assert!(gamma.is_finite());
        assert!(gamma >= 1.0);
    }

    // --- calculate_player_clock ---

    #[test]
    fn player_clock_advances_at_rest() {
        let dt = UomTime::new::<second>(1.0);
        let prev = UomTime::new::<second>(10.0);
        let result = calculate_player_clock(dt, 1.0, 1.0, prev);
        assert_relative_eq!(result.value, UomTime::new::<second>(11.0).value);
    }

    #[test]
    fn player_clock_slower_with_velocity_gamma() {
        let dt = UomTime::new::<second>(1.0);
        let prev = UomTime::new::<second>(0.0);
        let result_rest = calculate_player_clock(dt, 1.0, 1.0, prev);
        let result_moving = calculate_player_clock(dt, 2.0, 1.0, prev);
        assert!(result_rest.value > result_moving.value);
    }

    #[test]
    fn player_clock_slower_with_gravitational_gamma() {
        let dt = UomTime::new::<second>(1.0);
        let prev = UomTime::new::<second>(0.0);
        let result_flat = calculate_player_clock(dt, 1.0, 1.0, prev);
        let result_gravity = calculate_player_clock(dt, 1.0, 2.0, prev);
        assert!(result_flat.value > result_gravity.value);
    }

    #[test]
    fn player_clock_preserves_previous() {
        let dt = UomTime::new::<second>(0.0);
        let prev = UomTime::new::<second>(42.0);
        let result = calculate_player_clock(dt, 1.0, 1.0, prev);
        assert_relative_eq!(result.value, prev.value);
    }

    #[test]
    fn player_clock_combined_gamma_slows_more() {
        let dt = UomTime::new::<second>(1.0);
        let prev = UomTime::new::<second>(0.0);
        let result_v_only = calculate_player_clock(dt, 2.0, 1.0, prev);
        let result_both = calculate_player_clock(dt, 2.0, 2.0, prev);
        assert!(result_v_only.value > result_both.value);
    }

    // --- proptest property-based tests ---

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        /// Speed of light in km/s for range calculations.
        const C_KMS: f64 = 299_792.0;

        proptest! {
            #[test]
            fn velocity_gamma_ge_one_for_all_sub_c(
                vx_kms in -0.99 * C_KMS..0.99 * C_KMS,
                vy_kms in -0.99 * C_KMS..0.99 * C_KMS,
            ) {
                // Skip if combined speed >= c.
                let v_sq = vx_kms * vx_kms + vy_kms * vy_kms;
                let c_sq = C_KMS * C_KMS;
                prop_assume!(v_sq < 0.99 * 0.99 * c_sq);

                let gamma = calculate_velocity_gamma(make_vel(vx_kms), make_vel(vy_kms), c());
                prop_assert!(gamma >= 1.0, "gamma was {} for vx={}, vy={}", gamma, vx_kms, vy_kms);
                prop_assert!(gamma.is_finite(), "gamma was infinite for vx={}, vy={}", vx_kms, vy_kms);
            }

            #[test]
            fn gravitational_gamma_ge_one_for_positive_mass_and_distance(
                px_km in 1.0e6_f64..1.0e12,
                py_km in -1.0e12_f64..1.0e12,
                mass_kg in 1.0e20_f64..1.0e40,
            ) {
                let gamma = calculate_gravitational_gamma(
                    UomLength::new::<kilometer>(px_km),
                    UomLength::new::<kilometer>(py_km),
                    &[(UomLength::new::<kilometer>(0.0), UomLength::new::<kilometer>(0.0), UomMass::new::<kilogram>(mass_kg))],
                );
                prop_assert!(gamma >= 1.0, "gamma was {} for px={}, py={}, mass={}", gamma, px_km, py_km, mass_kg);
                prop_assert!(gamma.is_finite(), "gamma was infinite for px={}, py={}, mass={}", px_km, py_km, mass_kg);
            }
        }
    }
}
