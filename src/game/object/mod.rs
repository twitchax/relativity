use bevy::prelude::*;
use super::shared::{types::{Position, Mass, Radius}, helpers::get_position_from_percentage, constants::{UNIT_RADIUS, MASS_OF_SUN, MASS_OF_EARTH}};

#[derive(Component, Default)]
pub struct Planet;

#[derive(Bundle, Default)]
pub struct PlanetBundle {
    pub planet: Planet,
    pub position: Position,
    pub mass: Mass,
    pub radius: Radius,
    pub sprite: SpriteBundle,
}


pub fn spawn_planets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // SUN
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.5, 0.5),
        radius: Radius {
            value: 3.0 * *UNIT_RADIUS,
        },
        mass: Mass {
            value: *MASS_OF_SUN,
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet04.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // SUN2
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.8, 0.7),
        radius: Radius {
            value: 2.0 * *UNIT_RADIUS,
        },
        mass: Mass {
            value: 0.2 * *MASS_OF_SUN,
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet05.png"),
            ..Default::default()
        },
        ..Default::default()
    });


    // EARTH
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.28, 0.28),
        radius: Radius {
            value: 2.0 * *UNIT_RADIUS,
        },
        mass: Mass {
            value: *MASS_OF_EARTH,
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet03.png"),
            ..Default::default()
        },
        ..Default::default()
    });
}