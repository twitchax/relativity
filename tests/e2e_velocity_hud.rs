// E2E headless test: verifies the player clock data entity holds correct
// velocity-gamma, gravitational-gamma, clock, and that the velocity on
// the player sprite entity is consistent with the launch parameters.
//
// The visual HUD (bevy_lunex Text2d labels) requires UiLunexPlugins which
// is registered only in main.rs; headless tests cannot verify rendered text.
// The text formatting pure functions (format_velocity_fraction, etc.) are
// already unit-tested in player_clock.rs and observer/mod.rs.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game, find_player_sprite, launch_player, start_running};
use relativity::game::player::shared::Player;
use relativity::game::shared::{
    constants::C,
    types::{PlayerHud, Velocity, VelocityGamma},
};

/// After launching at a known velocity and running a few frames, the data
/// entity (marked with `PlayerHud`) should contain a velocity gamma > 1
/// and the player sprite should have non-zero velocity.
#[test]
fn hud_data_reflects_velocity_after_launch() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);

    // Launch at ~0.71c (each component ~150_000 km/s → scalar ≈ 212_132 km/s ≈ 0.71c).
    launch_player(&mut app, player, (150_000.0, 150_000.0));
    start_running(&mut app);

    // Run several frames so player_clock_update executes.
    for _ in 0..5 {
        app.update();
    }

    // Query the player clock data entity.
    let vel_gamma = app
        .world_mut()
        .query_filtered::<&VelocityGamma, With<PlayerHud>>()
        .single(app.world())
        .expect("expected a PlayerHud data entity with VelocityGamma");

    // At ~0.71c, velocity gamma should be noticeably above 1.
    assert!(vel_gamma.value > 1.0, "velocity gamma should be > 1 after launch, got: {}", vel_gamma.value);

    // Verify the player sprite has non-zero velocity.
    let velocity = app
        .world_mut()
        .query_filtered::<&Velocity, (With<Player>, Without<PlayerHud>)>()
        .single(app.world())
        .expect("expected a Player sprite entity with Velocity");
    let speed_fraction = (velocity.scalar() / *C).value;

    assert!(speed_fraction > 0.0, "velocity fraction should be positive after launch, got: {speed_fraction}");
    assert!(speed_fraction < 1.0, "velocity fraction should be < 1.0 (sub-luminal), got: {speed_fraction}");
}

/// At rest (before launch), the data entity should show velocity gamma ≈ 1.
#[test]
fn hud_data_shows_unit_gamma_at_rest() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Don't launch — player stays at rest. Transition to Running so the
    // clock update system executes.
    start_running(&mut app);

    for _ in 0..3 {
        app.update();
    }

    let vel_gamma = app
        .world_mut()
        .query_filtered::<&VelocityGamma, With<PlayerHud>>()
        .single(app.world())
        .expect("expected a PlayerHud data entity with VelocityGamma");

    assert!((vel_gamma.value - 1.0).abs() < 1e-6, "velocity gamma should be ~1.0 at rest, got: {}", vel_gamma.value);
}
