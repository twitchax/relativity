use super::shared::types::{GameItem, Mass, PlanetSprite, Position, Radius};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Destination;

#[derive(Bundle, Default)]
pub struct DestinationBundle {
    pub item: GameItem,
    pub destination: Destination,
    pub position: Position,
    pub mass: Mass,
    pub radius: Radius,
    pub sprite_type: PlanetSprite,
    pub sprite: SpriteBundle,
}
