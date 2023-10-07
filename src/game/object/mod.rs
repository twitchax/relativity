use super::shared::types::{Mass, Position, Radius, Velocity, GameItem};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Planet;

#[derive(Bundle, Default)]
pub struct StaticPlanetBundle {
    pub item: GameItem,
    pub planet: Planet,
    pub position: Position,
    pub mass: Mass,
    pub radius: Radius,
    pub sprite: SpriteBundle,
}

#[derive(Bundle, Default)]
pub struct DynamicPlanetBundle {
    pub item: GameItem,
    pub planet: Planet,
    pub position: Position,
    pub mass: Mass,
    pub radius: Radius,
    pub velocity: Velocity,
    pub sprite: SpriteBundle,
}