use bevy::prelude::*;

use super::{object::StaticPlanetBundle, shared::{helpers::get_position_from_percentage, constants::{UNIT_RADIUS, MASS_OF_SUN, MASS_OF_EARTH}, types::{Radius, Mass, GameItem}}, player::{player_clock::spawn_player_clock, player_sprite::PlayerSpriteBundle}, observer::spawn_observer_clock};

// Components / bundles / resources.

#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
}

// Startup systems.

pub fn spawn_level(commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    match current_level.into_inner() {
        CurrentLevel::One => level1(commands, asset_server)
    }
}

pub fn despawn_level(mut commands: Commands, query: Query<Entity, With<GameItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Levels.

pub fn level1(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn clocks.

    spawn_player_clock(&mut commands, &asset_server);
    spawn_observer_clock(&mut commands, &asset_server);

    // Spawn player.

    commands.spawn(PlayerSpriteBundle {
        position: get_position_from_percentage(0.3, 0.3),
        radius: Radius { value: *UNIT_RADIUS },
        sprite: SpriteBundle { texture: asset_server.load("sprites/planets/sphere2.png"), ..Default::default() },
        ..Default::default()
    });

    // Spawn objects.

    // SUN
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.5, 0.5),
        radius: Radius { value: 3.0 * *UNIT_RADIUS },
        mass: Mass { value: *MASS_OF_SUN },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet04.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // SUN2
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.8, 0.7),
        radius: Radius { value: 2.0 * *UNIT_RADIUS },
        mass: Mass { value: 0.4 * *MASS_OF_SUN },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet05.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // EARTH
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.28, 0.28),
        radius: Radius { value: 2.0 * *UNIT_RADIUS },
        mass: Mass { value: *MASS_OF_EARTH },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet03.png"),
            ..Default::default()
        },
        ..Default::default()
    });
}