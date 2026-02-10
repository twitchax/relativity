// E2E headless test: verifies that the player clock experiences time dilation
// when the player has nonzero velocity or is near a massive body.
//
// Uses MinimalPlugins with ManualDuration time strategy to ensure deterministic
// time advancement in a headless environment.
//
// The observer clock advances at the coordinate rate (no dilation), while the
// player clock advances slower due to velocity gamma > 1 and/or gravitational
// gamma > 1.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use relativity::game::observer::observer_clock_update;
use relativity::game::{
    observer::ObserverClockBundle,
    player::{
        player_clock::{player_clock_update, PlayerClockBundle},
        shared::Player,
    },
    shared::types::{Clock, Mass, Position, SimRate, Velocity},
};
use uom::si::{
    f64::{Length as UomLength, Mass as UomMass, Velocity as UomVelocity},
    length::kilometer,
    mass::kilogram,
    time::second,
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

/// Build a headless app with observer_clock_update and player_clock_update systems.
fn build_time_dilation_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .init_resource::<SimRate>()
        .add_systems(Update, (observer_clock_update, player_clock_update));
    app
}

/// Spawn the player sprite entity (Position + Velocity + Player).
/// This is the entity the player_clock_update system reads position/velocity from.
fn spawn_player_sprite(world: &mut World, pos_x: f64, pos_y: f64, vel_x: f64, vel_y: f64) -> Entity {
    world.spawn((Player, Position { x: km(pos_x), y: km(pos_y) }, Velocity { x: kms(vel_x), y: kms(vel_y) })).id()
}

/// Spawn the player clock entity (Player + Clock + VelocityGamma + GravitationalGamma).
/// This is the entity the player_clock_update system writes clock/gamma values to.
fn spawn_player_clock(world: &mut World) -> Entity {
    world.spawn(PlayerClockBundle::default()).id()
}

/// Spawn the observer clock entity (Observer + Clock).
fn spawn_observer_clock(world: &mut World) -> Entity {
    world.spawn(ObserverClockBundle::default()).id()
}

/// Spawn a massive body at a given position.
fn spawn_mass(world: &mut World, pos_x: f64, pos_y: f64, mass: f64) -> Entity {
    world.spawn((Position { x: km(pos_x), y: km(pos_y) }, Mass { value: kg(mass) })).id()
}

/// Read the clock value in seconds from an entity.
fn read_clock_seconds(world: &World, entity: Entity) -> f64 {
    world.get::<Clock>(entity).unwrap().value.get::<second>()
}

// ---------- Velocity time dilation ----------

#[test]
fn player_clock_slower_than_observer_with_velocity() {
    let mut app = build_time_dilation_app();

    // Player moving at ~90% of c => significant velocity gamma.
    spawn_player_sprite(app.world_mut(), 0.0, 0.0, 269_813.0, 0.0);
    let player_clock = spawn_player_clock(app.world_mut());
    let observer_clock = spawn_observer_clock(app.world_mut());

    // Run several frames to accumulate clock values.
    for _ in 0..30 {
        app.update();
    }

    let player_time = read_clock_seconds(app.world(), player_clock);
    let observer_time = read_clock_seconds(app.world(), observer_clock);

    assert!(observer_time > 0.0, "observer clock should advance: {observer_time}");
    assert!(player_time > 0.0, "player clock should advance: {player_time}");
    assert!(
        player_time < observer_time,
        "player clock should run slower than observer due to velocity time dilation: player={player_time}, observer={observer_time}"
    );
}

#[test]
fn player_clock_equals_observer_at_rest() {
    let mut app = build_time_dilation_app();

    // Player at rest with no masses => no time dilation (gamma = 1).
    spawn_player_sprite(app.world_mut(), 0.0, 0.0, 0.0, 0.0);
    let player_clock = spawn_player_clock(app.world_mut());
    let observer_clock = spawn_observer_clock(app.world_mut());

    for _ in 0..30 {
        app.update();
    }

    let player_time = read_clock_seconds(app.world(), player_clock);
    let observer_time = read_clock_seconds(app.world(), observer_clock);

    assert!(observer_time > 0.0, "observer clock should advance: {observer_time}");
    approx::assert_relative_eq!(player_time, observer_time, epsilon = 1e-10);
}

#[test]
fn faster_velocity_means_more_dilation() {
    // Two runs: slow and fast velocity. Faster should have more dilation (smaller player_time / observer_time ratio).
    let run = |vel_kms: f64| -> (f64, f64) {
        let mut app = build_time_dilation_app();
        spawn_player_sprite(app.world_mut(), 0.0, 0.0, vel_kms, 0.0);
        let player_clock = spawn_player_clock(app.world_mut());
        let observer_clock = spawn_observer_clock(app.world_mut());

        for _ in 0..30 {
            app.update();
        }

        let player_time = read_clock_seconds(app.world(), player_clock);
        let observer_time = read_clock_seconds(app.world(), observer_clock);
        (player_time, observer_time)
    };

    let (slow_player, slow_observer) = run(100_000.0); // ~33% of c
    let (fast_player, fast_observer) = run(269_813.0); // ~90% of c

    let slow_ratio = slow_player / slow_observer;
    let fast_ratio = fast_player / fast_observer;

    assert!(fast_ratio < slow_ratio, "faster velocity should cause more dilation: slow_ratio={slow_ratio}, fast_ratio={fast_ratio}");
}

#[test]
fn gravitational_time_dilation_near_massive_body() {
    let mut app = build_time_dilation_app();

    // Player at rest near a very massive body => gravitational gamma > 1.
    spawn_player_sprite(app.world_mut(), 1_000_000.0, 0.0, 0.0, 0.0);
    let player_clock = spawn_player_clock(app.world_mut());
    let observer_clock = spawn_observer_clock(app.world_mut());

    // Use a very large mass to get measurable gravitational time dilation.
    // Real solar mass * MASS_FACTOR at moderate distance.
    spawn_mass(app.world_mut(), 0.0, 0.0, 1.989e38);

    for _ in 0..30 {
        app.update();
    }

    let player_time = read_clock_seconds(app.world(), player_clock);
    let observer_time = read_clock_seconds(app.world(), observer_clock);

    assert!(observer_time > 0.0, "observer clock should advance: {observer_time}");
    assert!(player_time > 0.0, "player clock should advance: {player_time}");
    assert!(
        player_time < observer_time,
        "player clock should run slower near massive body due to gravitational dilation: player={player_time}, observer={observer_time}"
    );
}

#[test]
fn combined_velocity_and_gravity_dilation() {
    // Player with both velocity AND near a mass should have MORE dilation than either alone.
    let run = |vel_kms: f64, add_mass: bool| -> f64 {
        let mut app = build_time_dilation_app();
        spawn_player_sprite(app.world_mut(), 1_000_000.0, 0.0, vel_kms, 0.0);
        let player_clock = spawn_player_clock(app.world_mut());
        let observer_clock = spawn_observer_clock(app.world_mut());

        if add_mass {
            spawn_mass(app.world_mut(), 0.0, 0.0, 1.989e38);
        }

        for _ in 0..30 {
            app.update();
        }

        let player_time = read_clock_seconds(app.world(), player_clock);
        let observer_time = read_clock_seconds(app.world(), observer_clock);
        player_time / observer_time
    };

    let ratio_velocity_only = run(200_000.0, false);
    let ratio_gravity_only = run(0.0, true);
    let ratio_both = run(200_000.0, true);

    assert!(
        ratio_both < ratio_velocity_only,
        "combined dilation should exceed velocity-only: both={ratio_both}, velocity_only={ratio_velocity_only}"
    );
    assert!(
        ratio_both < ratio_gravity_only,
        "combined dilation should exceed gravity-only: both={ratio_both}, gravity_only={ratio_gravity_only}"
    );
}
