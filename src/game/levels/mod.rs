use bevy::prelude::*;

use super::{
    destination::DestinationBundle,
    object::StaticPlanetBundle,
    observer::spawn_observer_clock,
    player::{player_clock::spawn_player_clock, player_sprite::PlayerSpriteBundle},
    shared::{
        constants::{MASS_OF_EARTH, MASS_OF_SUN, UNIT_RADIUS},
        helpers::get_position_from_percentage,
        types::{GameItem, Mass, Radius},
    },
};

// Components / bundles / resources.

#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
}

// Startup systems.

pub fn spawn_level(commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    match current_level.into_inner() {
        CurrentLevel::One => level1(commands, asset_server),
    }
}

pub fn despawn_level(mut commands: Commands, query: Query<Entity, With<GameItem>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
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
        radius: Radius { value: *UNIT_RADIUS / 4.0 },
        sprite: Sprite {
            image: asset_server.load("sprites/space/Rockets/spaceRockets_003.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn objects.

    // SUN
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.5, 0.5),
        radius: Radius { value: 3.0 * *UNIT_RADIUS },
        mass: Mass { value: *MASS_OF_SUN },
        sprite: Sprite {
            image: asset_server.load("sprites/planets/planet04.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // SUN2
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.8, 0.7),
        radius: Radius { value: 2.0 * *UNIT_RADIUS },
        mass: Mass { value: 0.4 * *MASS_OF_SUN },
        sprite: Sprite {
            image: asset_server.load("sprites/planets/planet05.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // EARTH
    commands.spawn(StaticPlanetBundle {
        position: get_position_from_percentage(0.28, 0.28),
        radius: Radius { value: 2.0 * *UNIT_RADIUS },
        mass: Mass { value: *MASS_OF_EARTH },
        sprite: Sprite {
            image: asset_server.load("sprites/planets/planet03.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn destination.

    commands.spawn(DestinationBundle {
        position: get_position_from_percentage(0.9, 0.9),
        radius: Radius { value: 4.0 * *UNIT_RADIUS },
        mass: Mass { value: 0.6 * *MASS_OF_SUN },
        sprite: Sprite {
            image: asset_server.load("sprites/planets/noise00.png"),
            ..Default::default()
        },
        ..Default::default()
    });
}
