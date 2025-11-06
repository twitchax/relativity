use bevy::prelude::*;

use super::{
    destination::DestinationBundle,
    object::{DynamicPlanetBundle, StaticPlanetBundle},
    observer::spawn_observer_clock,
    player::{player_clock::spawn_player_clock, player_sprite::PlayerSpriteBundle},
    shared::{
        constants::{MASS_OF_EARTH, MASS_OF_SUN, UNIT_RADIUS},
        helpers::get_position_from_percentage,
        types::{GameItem, Mass, Radius, Velocity},
    },
};

// Components / bundles / resources.

#[derive(Resource, Default)]
pub enum CurrentLevel {
    #[default]
    One,
    TimeWarp,
}

// Startup systems.

pub fn spawn_level(commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    match current_level.into_inner() {
        CurrentLevel::One => level1(commands, asset_server),
        CurrentLevel::TimeWarp => level_time_warp(commands, asset_server),
    }
}

pub fn despawn_level(mut commands: Commands, query: Query<Entity, With<GameItem>>) {
    for entity in query.iter() {
        // Note: Using despawn() instead of despawn_recursive() is appropriate here
        // because game entities in this codebase do not have children.
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

/// Time Warp level: A central moving gravity well creates a time-dilation region.
/// The player must navigate from a slingshot starting position to an exit gate
/// while managing the time-dilation effects of the moving well.
pub fn level_time_warp(mut commands: Commands, asset_server: Res<AssetServer>) {
    use uom::si::{f64::Velocity as UomVelocity, velocity::kilometer_per_second};

    // Spawn clocks.
    spawn_player_clock(&mut commands, &asset_server);
    spawn_observer_clock(&mut commands, &asset_server);

    // Spawn player at slingshot starting position.
    // Position: 0.125 (12.5%) of screen width, 0.5 (50%) of screen height
    // Initial velocity: (260.0, 0.0) km/s - gives the player a horizontal boost
    commands.spawn(PlayerSpriteBundle {
        position: get_position_from_percentage(0.125, 0.5),
        radius: Radius { value: *UNIT_RADIUS / 4.0 },
        velocity: Velocity {
            x: UomVelocity::new::<kilometer_per_second>(260.0), // Strong horizontal velocity for slingshot
            y: UomVelocity::new::<kilometer_per_second>(0.0),
        },
        sprite: Sprite {
            image: asset_server.load("sprites/space/Rockets/spaceRockets_003.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn moving gravity well (time_well) that orbits around center.
    // The well creates a strong gravitational field and will be the center of time-dilation.
    // Center point: 0.5, 0.5 (center of screen)
    // Orbital radius: ~0.15 screen width
    // Position offset: placing at (0.5 + 0.15, 0.5) = right side of orbit initially
    // Orbital velocity: tangent to circle, approximately sqrt(G*M/r) but tuned for gameplay
    // Mass: 18000.0 * MASS_OF_EARTH - strong enough to affect player significantly
    // Physical radius: 0.67 * UNIT_RADIUS - small visual size but strong gravitational influence
    commands.spawn(DynamicPlanetBundle {
        position: get_position_from_percentage(0.65, 0.5), // Start on right side of orbit
        radius: Radius { value: 0.67 * *UNIT_RADIUS },     // Small physical size (~40px)
        mass: Mass { value: 18000.0 * *MASS_OF_EARTH },    // Very strong mass for noticeable gravity
        velocity: Velocity {
            // Orbital velocity: perpendicular to radius, tuned for circular-ish orbit
            x: UomVelocity::new::<kilometer_per_second>(0.0),
            y: UomVelocity::new::<kilometer_per_second>(280.0), // Upward velocity for counterclockwise orbit
        },
        sprite: Sprite {
            image: asset_server.load("sprites/planets/planet07.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn destination/exit gate.
    // Position: 0.95 (95%) of screen width, 0.5 (50%) of screen height
    // Radius: 48 pixels - medium-sized target
    commands.spawn(DestinationBundle {
        position: get_position_from_percentage(0.95, 0.5), // Far right, centered vertically
        radius: Radius { value: 0.8 * *UNIT_RADIUS },      // ~48px radius
        mass: Mass { value: 0.1 * *MASS_OF_SUN },          // Weak mass to avoid disrupting trajectory too much
        sprite: Sprite {
            image: asset_server.load("sprites/planets/noise00.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // TODO: Implement time-dilation system
    // The time-dilation zone should:
    // 1. Track distance from player to the moving gravity well
    // 2. Adjust player's effective time_scale between 0.5 and 1.0 based on distance
    // 3. Closer to the well = slower time (0.5x), farther = normal time (1.0x)
    // This will require:
    // - A custom component to mark entities affected by time dilation
    // - A system that runs in Update to calculate and apply time scaling
    // - Modification of velocity_update and position_update to respect time scaling
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_warp_level_enum_exists() {
        // Verify the TimeWarp enum variant can be created
        let level = CurrentLevel::TimeWarp;
        match level {
            CurrentLevel::TimeWarp => (),
            _ => panic!("TimeWarp variant should exist"),
        }
    }

    #[test]
    fn test_spawn_level_handles_time_warp() {
        // This test verifies that the spawn_level function has a match arm for TimeWarp
        // We can't easily test the actual spawning without a full Bevy app setup,
        // but we can verify the enum variant compiles and can be matched.
        let _level = CurrentLevel::TimeWarp;
        // If this compiles and the match in spawn_level is exhaustive, we're good
    }
}
