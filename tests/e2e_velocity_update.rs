// E2E headless test: verifies that velocity_update applies gravitational
// acceleration correctly — a player near a massive body should gain velocity
// toward that body over several frames.
//
// Uses MinimalPlugins with ManualDuration time strategy to ensure deterministic
// time advancement in a headless environment.
//
// NOTE: Mass and distance values must be chosen so that the relativistic
// adjustment (1 − 2GM/c²d) remains positive; otherwise the system clamps
// acceleration to zero.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use relativity::game::shared::{
    systems::velocity_update,
    types::{Mass, Position, SimRate, Velocity},
};
use uom::si::{
    f64::{Length as UomLength, Mass as UomMass, Velocity as UomVelocity},
    length::kilometer,
    mass::kilogram,
    velocity::kilometer_per_second,
};

fn km(v: f64) -> UomLength {
    UomLength::new::<kilometer>(v)
}

fn kg(v: f64) -> UomMass {
    UomMass::new::<kilogram>(v)
}

fn kms(v: f64) -> UomVelocity {
    UomVelocity::new::<kilometer_per_second>(v)
}

// Real solar mass (no MASS_FACTOR) at large distances to avoid relativistic clamp.
const TEST_MASS: f64 = 1.989e30;
const TEST_DISTANCE: f64 = 1_000_000.0; // km

/// Build a headless app with the velocity_update system and a fixed time step.
fn build_velocity_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .init_resource::<SimRate>()
        .add_systems(Update, velocity_update);
    app
}

/// Spawn a player entity (has Position + Velocity, no Mass so it is not a gravitational source).
fn spawn_player(world: &mut World, pos_x: f64, pos_y: f64, vel_x: f64, vel_y: f64) -> Entity {
    world.spawn((Position { x: km(pos_x), y: km(pos_y) }, Velocity { x: kms(vel_x), y: kms(vel_y) })).id()
}

/// Spawn a massive body (has Position + Mass, acts as a gravitational source).
fn spawn_mass(world: &mut World, pos_x: f64, pos_y: f64, mass: f64) -> Entity {
    world.spawn((Position { x: km(pos_x), y: km(pos_y) }, Mass { value: kg(mass) })).id()
}

// ---------- Gravity accelerates player toward planet ----------

#[test]
fn player_velocity_increases_toward_planet() {
    let mut app = build_velocity_test_app();

    // Player at origin with small initial velocity (both axes non-zero to pass the guard).
    // Planet to the right with real solar mass at safe distance.
    let player = spawn_player(app.world_mut(), 0.0, 0.0, 0.001, 0.001);
    spawn_mass(app.world_mut(), TEST_DISTANCE, 0.0, TEST_MASS);

    // First update has delta=0, subsequent ones advance by 1/60s.
    for _ in 0..5 {
        app.update();
    }

    let initial_vx = app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>();

    for _ in 0..20 {
        app.update();
    }

    let final_vx = app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>();

    assert!(final_vx > initial_vx, "player should accelerate toward planet: initial vx={initial_vx}, final vx={final_vx}");
}

#[test]
fn player_velocity_y_increases_toward_planet_above() {
    let mut app = build_velocity_test_app();

    // Planet above player (positive y).
    let player = spawn_player(app.world_mut(), 0.0, 0.0, 0.001, 0.001);
    spawn_mass(app.world_mut(), 0.0, TEST_DISTANCE, TEST_MASS);

    for _ in 0..5 {
        app.update();
    }

    let initial_vy = app.world().get::<Velocity>(player).unwrap().y.get::<kilometer_per_second>();

    for _ in 0..20 {
        app.update();
    }

    let final_vy = app.world().get::<Velocity>(player).unwrap().y.get::<kilometer_per_second>();

    assert!(final_vy > initial_vy, "player should accelerate toward planet above: initial vy={initial_vy}, final vy={final_vy}");
}

#[test]
fn no_velocity_change_without_mass() {
    let mut app = build_velocity_test_app();

    // Player with initial velocity but no masses in the world.
    let player = spawn_player(app.world_mut(), 0.0, 0.0, 1.0, 1.0);

    for _ in 0..5 {
        app.update();
    }

    let initial_vx = app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>();

    for _ in 0..20 {
        app.update();
    }

    let final_vx = app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>();

    approx::assert_relative_eq!(initial_vx, final_vx, epsilon = 1e-15);
}

#[test]
fn heavier_mass_produces_greater_acceleration() {
    // Two separate runs: one with a lighter mass, one with a heavier mass.
    // Both must avoid the relativistic clamp (stay well below Schwarzschild radius).
    let run = |mass: f64| -> f64 {
        let mut app = build_velocity_test_app();
        let player = spawn_player(app.world_mut(), 0.0, 0.0, 0.001, 0.001);
        spawn_mass(app.world_mut(), TEST_DISTANCE, 0.0, mass);

        for _ in 0..30 {
            app.update();
        }

        app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>()
    };

    let vx_light = run(1.989e28); // 1/100th of solar mass
    let vx_heavy = run(1.989e30); // real solar mass

    assert!(vx_heavy > vx_light, "heavier mass should produce greater acceleration: light vx={vx_light}, heavy vx={vx_heavy}");
}

#[test]
fn zero_velocity_axis_skips_gravity() {
    let mut app = build_velocity_test_app();

    // velocity.x == 0.0, so the system skips this entity (guard clause uses ||).
    let player = spawn_player(app.world_mut(), 0.0, 0.0, 0.0, 1.0);
    spawn_mass(app.world_mut(), TEST_DISTANCE, 0.0, TEST_MASS);

    for _ in 0..20 {
        app.update();
    }

    let final_vx = app.world().get::<Velocity>(player).unwrap().x.get::<kilometer_per_second>();

    // Should remain at zero — the guard clause prevents gravity from applying.
    approx::assert_relative_eq!(final_vx, 0.0, epsilon = 1e-15);
}
