//! Shared test utilities for headless Bevy ECS tests.
//!
//! Provides helpers to build a `MinimalPlugins` app, spawn common game entities,
//! and advance the app by N frames.

use bevy::prelude::*;
use uom::si::{
    f64::{Length as UomLength, Mass as UomMass, Velocity as UomVelocity},
    length::kilometer,
    mass::kilogram,
    velocity::kilometer_per_second,
};

use super::shared::types::{Mass, Position, Radius, Velocity};

/// Build a headless Bevy `App` with `MinimalPlugins` plus `TransformPlugin`.
///
/// This provides time, scheduling, and transform propagation — enough
/// infrastructure to run most game systems without a GPU or window.
pub fn minimal_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(TransformPlugin);
    app
}

/// Advance the app by `n` update cycles.
pub fn run_frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

/// Spawn a test entity with `Position`, `Velocity`, `Mass`, and `Radius` components.
///
/// Values are specified in convenient units (km, km/s, kg, km) and converted to
/// the UOM types used by the game.
#[allow(clippy::similar_names)]
pub fn spawn_test_entity(world: &mut World, pos_x_km: f64, pos_y_km: f64, vel_x_kms: f64, vel_y_kms: f64, mass_kg: f64, radius_km: f64) -> Entity {
    world
        .spawn((
            Position {
                x: UomLength::new::<kilometer>(pos_x_km),
                y: UomLength::new::<kilometer>(pos_y_km),
            },
            Velocity {
                x: UomVelocity::new::<kilometer_per_second>(vel_x_kms),
                y: UomVelocity::new::<kilometer_per_second>(vel_y_kms),
            },
            Mass { value: UomMass::new::<kilogram>(mass_kg) },
            Radius {
                value: UomLength::new::<kilometer>(radius_km),
            },
        ))
        .id()
}

/// Spawn a test entity at a position with zero velocity and no mass/radius.
///
/// Convenient for entities that only need spatial placement.
#[allow(clippy::similar_names)]
pub fn spawn_positioned_entity(world: &mut World, pos_x_km: f64, pos_y_km: f64) -> Entity {
    spawn_test_entity(world, pos_x_km, pos_y_km, 0.0, 0.0, 0.0, 0.0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uom::si::{length::kilometer, mass::kilogram, velocity::kilometer_per_second};

    #[test]
    fn minimal_test_app_creates_app_with_time() {
        let mut app = minimal_test_app();
        // Should be able to run at least one update without panicking.
        app.update();
        // Time resource should exist.
        assert!(app.world().get_resource::<Time>().is_some());
    }

    #[test]
    fn run_frames_advances_multiple_updates() {
        let mut app = minimal_test_app();
        // Should not panic when running multiple frames.
        run_frames(&mut app, 5);
    }

    #[test]
    fn spawn_test_entity_has_all_components() {
        let mut app = minimal_test_app();
        let entity = spawn_test_entity(app.world_mut(), 100.0, 200.0, 10.0, 20.0, 1.0e30, 500.0);

        let world = app.world();
        let pos = world.get::<Position>(entity).unwrap();
        let vel = world.get::<Velocity>(entity).unwrap();
        let mass = world.get::<Mass>(entity).unwrap();
        let radius = world.get::<Radius>(entity).unwrap();

        approx::assert_relative_eq!(pos.x.get::<kilometer>(), 100.0);
        approx::assert_relative_eq!(pos.y.get::<kilometer>(), 200.0);
        approx::assert_relative_eq!(vel.x.get::<kilometer_per_second>(), 10.0);
        approx::assert_relative_eq!(vel.y.get::<kilometer_per_second>(), 20.0);
        approx::assert_relative_eq!(mass.value.get::<kilogram>(), 1.0e30);
        approx::assert_relative_eq!(radius.value.get::<kilometer>(), 500.0);
    }

    #[test]
    fn spawn_positioned_entity_has_zero_velocity_and_mass() {
        let mut app = minimal_test_app();
        let entity = spawn_positioned_entity(app.world_mut(), 50.0, -75.0);

        let world = app.world();
        let pos = world.get::<Position>(entity).unwrap();
        let vel = world.get::<Velocity>(entity).unwrap();
        let mass = world.get::<Mass>(entity).unwrap();
        let radius = world.get::<Radius>(entity).unwrap();

        approx::assert_relative_eq!(pos.x.get::<kilometer>(), 50.0);
        approx::assert_relative_eq!(pos.y.get::<kilometer>(), -75.0);
        approx::assert_relative_eq!(vel.x.get::<kilometer_per_second>(), 0.0);
        approx::assert_relative_eq!(vel.y.get::<kilometer_per_second>(), 0.0);
        approx::assert_relative_eq!(mass.value.get::<kilogram>(), 0.0);
        approx::assert_relative_eq!(radius.value.get::<kilometer>(), 0.0);
    }
}
