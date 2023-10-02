use std::char::MAX;

use bevy::{input::keyboard, math, prelude::*, render::view::window, transform, window::PrimaryWindow};
use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration as UomAcceleration, Force as UomForce, Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity},
    force::newton,
    heat_capacity::gram_square_meter_per_second_squared_kelvin,
    length::{kilometer, meter},
    mass::kilogram,
    time::{day, hour, second},
    velocity::{kilometer_per_second, meter_per_second},
};

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub position: Position,
    pub radius: Radius,
    pub velocity: Velocity,
    pub sprite: SpriteBundle,
}

#[derive(Bundle, Default)]
pub struct PlanetBundle {
    pub planet: Planet,
    pub position: Position,
    pub mass: Mass,
    pub radius: Radius,
    pub sprite: SpriteBundle,
}

#[derive(Bundle, Default)]
pub struct PlayerClockBundle {
    pub player: Player,
    pub velocity_gamma: VelocityGamma,
    pub gravitational_gamma: GravitationalGamma,
    pub clock: Clock,
    pub clock_text: TextBundle,
}

#[derive(Bundle, Default)]
pub struct ObserverClockBundle {
    pub observer: Observer,
    pub clock: Clock,
    pub clock_text: TextBundle,
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Component, Default)]
pub struct Planet;

#[derive(Component, Default)]
pub struct Observer;

#[derive(Component, Default)]
pub struct Position {
    pub x: UomLength,
    pub y: UomLength,
}

#[derive(Component, Default)]
pub struct Radius {
    pub value: UomLength,
}

#[derive(Component, Default)]
pub struct Mass {
    pub value: UomMass,
}

#[derive(Component, Default)]
pub struct Velocity {
    pub x: UomVelocity,
    pub y: UomVelocity,
}

impl Velocity {
    pub fn scalar(&self) -> UomVelocity {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Component, Default)]
pub struct Clock {
    pub value: UomTime,
}

#[derive(Component, Default)]
pub struct VelocityGamma {
    pub value: f64,
}

#[derive(Component, Default)]
pub struct GravitationalGamma {
    pub value: f64,
}