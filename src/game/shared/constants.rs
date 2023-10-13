use once_cell::sync::Lazy;
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

pub static DAYS_PER_SECOND_UOM: Lazy<UomTime> = Lazy::new(|| UomTime::new::<day>(DAYS_PER_SECOND));
pub static UNIT_RADIUS: Lazy<UomLength> = Lazy::new(|| UomLength::new::<kilometer>(UNIT_RADIUS_KM));
pub static MASS_OF_SUN: Lazy<UomMass> = Lazy::new(|| UomMass::new::<kilogram>(MASS_OF_SUN_KG));
pub static MASS_OF_EARTH: Lazy<UomMass> = Lazy::new(|| UomMass::new::<kilogram>(MASS_OF_EARTH_KG));
pub static SCREEN_WIDTH_UOM: Lazy<UomLength> = Lazy::new(|| UomLength::new::<kilometer>(SCREEN_WIDTH_KM));
pub static SCREEN_HEIGHT_UOM: Lazy<UomLength> = Lazy::new(|| UomLength::new::<kilometer>(SCREEN_HEIGHT_KM));
pub static C: Lazy<UomVelocity> = Lazy::new(|| UomVelocity::new::<kilometer_per_second>(C_KMS));

// TODO: Fix this insanity, lol.
#[allow(clippy::type_complexity)]
pub static G: Lazy<
    uom::si::Quantity<
        (dyn uom::si::Dimension<
            I = uom::typenum::Z0,
            J = uom::typenum::Z0,
            Kind = (dyn uom::Kind + 'static),
            L = uom::typenum::PInt<uom::typenum::UInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>, uom::typenum::B1>>,
            M = uom::typenum::NInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>>,
            N = uom::typenum::Z0,
            T = uom::typenum::NInt<uom::typenum::UInt<uom::typenum::UInt<uom::typenum::UTerm, uom::typenum::B1>, uom::typenum::B0>>,
            Th = uom::typenum::Z0,
        > + 'static),
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
> = Lazy::new(|| {
    GRAVITATIONAL_CONSTANT * UomForce::new::<newton>(1.0) * UomLength::new::<meter>(1.0) * UomLength::new::<meter>(1.0)
        / (UomMass::new::<kilogram>(1.0) * UomMass::new::<kilogram>(1.0))
});

// TODO: Make this go away, and use some acceleration.
pub static MAX_PLAYER_LAUNCH_VELOCITY: Lazy<UomVelocity> = Lazy::new(|| UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS));
