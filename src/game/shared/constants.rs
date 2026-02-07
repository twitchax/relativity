use std::sync::LazyLock;
use uom::si::{
    f64::{Force as UomForce, Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity},
    force::newton,
    length::{kilometer, meter},
    mass::kilogram,
    time::day,
    velocity::kilometer_per_second,
};

use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

pub const PLANET_SPRITE_WIDTH_PX: f64 = 1280.0f64;
pub const ROCKET_SPRITE_WIDTH_PX: f64 = 234.0f64;

const MASS_FACTOR: f64 = 100_000_000.0f64;
const DAYS_PER_SECOND: f64 = 0.1f64;
const GRAVITATIONAL_CONSTANT: f64 = 6.674e-11f64;

const UNIT_RADIUS_KM: f64 = 60_000_000.0f64;
const MASS_OF_SUN_KG: f64 = MASS_FACTOR * 1.989e30f64;
const MASS_OF_EARTH_KG: f64 = MASS_FACTOR * 5.972e24f64;
const SCREEN_WIDTH_KM: f64 = 6_000_000_000.0f64;
const SCREEN_HEIGHT_KM: f64 = SCREEN_WIDTH_KM * SCREEN_HEIGHT_PX / SCREEN_WIDTH_PX;
const C_KMS: f64 = 299_792.0f64; // Speed of light in km/s.
const MAX_PLAYER_VELOCITY_KMS: f64 = 0.99 * C_KMS; // 99% of c.

pub static DAYS_PER_SECOND_UOM: LazyLock<UomTime> = LazyLock::new(|| UomTime::new::<day>(DAYS_PER_SECOND));
pub static UNIT_RADIUS: LazyLock<UomLength> = LazyLock::new(|| UomLength::new::<kilometer>(UNIT_RADIUS_KM));
pub static MASS_OF_SUN: LazyLock<UomMass> = LazyLock::new(|| UomMass::new::<kilogram>(MASS_OF_SUN_KG));
pub static MASS_OF_EARTH: LazyLock<UomMass> = LazyLock::new(|| UomMass::new::<kilogram>(MASS_OF_EARTH_KG));
pub static SCREEN_WIDTH_UOM: LazyLock<UomLength> = LazyLock::new(|| UomLength::new::<kilometer>(SCREEN_WIDTH_KM));
pub static SCREEN_HEIGHT_UOM: LazyLock<UomLength> = LazyLock::new(|| UomLength::new::<kilometer>(SCREEN_HEIGHT_KM));
pub static C: LazyLock<UomVelocity> = LazyLock::new(|| UomVelocity::new::<kilometer_per_second>(C_KMS));

// TODO: Fix this insanity, lol.
#[allow(clippy::type_complexity)]
pub static G: LazyLock<
    uom::si::Quantity<
        dyn uom::si::Dimension<
                I = uom::typenum::Z0,
                J = uom::typenum::Z0,
                Kind = dyn uom::Kind + 'static,
                L = uom::typenum::PInt<uom::typenum::UInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>, uom::typenum::B1>>,
                M = uom::typenum::NInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>>,
                N = uom::typenum::Z0,
                T = uom::typenum::NInt<uom::typenum::UInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>, uom::typenum::B0>>,
                Th = uom::typenum::Z0,
            > + 'static,
        dyn uom::si::Units<
            f64,
            amount_of_substance = uom::si::amount_of_substance::mole,
            electric_current = uom::si::electric_current::ampere,
            length = uom::si::length::meter,
            luminous_intensity = uom::si::luminous_intensity::candela,
            mass = uom::si::mass::kilogram,
            thermodynamic_temperature = uom::si::thermodynamic_temperature::kelvin,
            time = uom::si::time::second,
        >,
        f64,
    >,
> = LazyLock::new(|| {
    GRAVITATIONAL_CONSTANT * UomForce::new::<newton>(1.0) * UomLength::new::<meter>(1.0) * UomLength::new::<meter>(1.0) / (UomMass::new::<kilogram>(1.0) * UomMass::new::<kilogram>(1.0))
});

// TODO: Make this go away, and use some acceleration.
pub static MAX_PLAYER_LAUNCH_VELOCITY: LazyLock<UomVelocity> = LazyLock::new(|| UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS));

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uom::si::{length::kilometer as km_unit, mass::kilogram as kg_unit, time::day as day_unit, velocity::kilometer_per_second as kps_unit};

    // --- LazyLock initialization ---

    #[test]
    fn days_per_second_uom_initializes() {
        let val = DAYS_PER_SECOND_UOM.get::<day_unit>();
        assert!(val > 0.0, "DAYS_PER_SECOND_UOM should be positive");
    }

    #[test]
    fn unit_radius_initializes() {
        let val = UNIT_RADIUS.get::<km_unit>();
        assert!(val > 0.0, "UNIT_RADIUS should be positive");
    }

    #[test]
    fn mass_of_sun_initializes() {
        let val = MASS_OF_SUN.get::<kg_unit>();
        assert!(val > 0.0, "MASS_OF_SUN should be positive");
    }

    #[test]
    fn mass_of_earth_initializes() {
        let val = MASS_OF_EARTH.get::<kg_unit>();
        assert!(val > 0.0, "MASS_OF_EARTH should be positive");
    }

    #[test]
    fn screen_width_uom_initializes() {
        let val = SCREEN_WIDTH_UOM.get::<km_unit>();
        assert!(val > 0.0, "SCREEN_WIDTH_UOM should be positive");
    }

    #[test]
    fn screen_height_uom_initializes() {
        let val = SCREEN_HEIGHT_UOM.get::<km_unit>();
        assert!(val > 0.0, "SCREEN_HEIGHT_UOM should be positive");
    }

    #[test]
    fn c_initializes() {
        let val = C.get::<kps_unit>();
        assert!(val > 0.0, "C should be positive");
    }

    #[test]
    fn g_initializes_and_is_positive() {
        let val = G.value;
        assert!(val > 0.0, "G should be positive");
    }

    #[test]
    fn max_player_launch_velocity_initializes() {
        let val = MAX_PLAYER_LAUNCH_VELOCITY.get::<kps_unit>();
        assert!(val > 0.0, "MAX_PLAYER_LAUNCH_VELOCITY should be positive");
    }

    // --- Physical constant validation ---

    #[test]
    fn mass_of_sun_greater_than_mass_of_earth() {
        assert!(MASS_OF_SUN.get::<kg_unit>() > MASS_OF_EARTH.get::<kg_unit>(), "Sun mass should exceed Earth mass");
    }

    #[test]
    fn max_velocity_less_than_c() {
        assert!(
            MAX_PLAYER_LAUNCH_VELOCITY.get::<kps_unit>() < C.get::<kps_unit>(),
            "Max player velocity should be less than the speed of light"
        );
    }

    #[test]
    fn max_velocity_is_ninety_nine_percent_of_c() {
        let ratio = MAX_PLAYER_LAUNCH_VELOCITY.get::<kps_unit>() / C.get::<kps_unit>();
        approx::assert_relative_eq!(ratio, 0.99, epsilon = 1e-10);
    }

    #[test]
    fn screen_height_less_than_screen_width() {
        assert!(
            SCREEN_HEIGHT_UOM.get::<km_unit>() < SCREEN_WIDTH_UOM.get::<km_unit>(),
            "Screen height should be less than screen width (landscape aspect ratio)"
        );
    }

    const _: () = assert!(PLANET_SPRITE_WIDTH_PX > 0.0);
    const _: () = assert!(ROCKET_SPRITE_WIDTH_PX > 0.0);
    const _: () = assert!(PLANET_SPRITE_WIDTH_PX > ROCKET_SPRITE_WIDTH_PX);

    #[test]
    fn gravitational_constant_matches_known_value() {
        approx::assert_relative_eq!(G.value, 6.674e-11, epsilon = 1e-14);
    }

    #[test]
    fn c_matches_known_speed_of_light() {
        approx::assert_relative_eq!(C.get::<kps_unit>(), 299_792.0, epsilon = 1.0);
    }
}
