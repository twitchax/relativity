use bevy::prelude::*;
use uom::si::f64::{Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity};

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