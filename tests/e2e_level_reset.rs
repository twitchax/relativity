// E2E headless test: verifies that after the player fails (planet collision),
// the entire level resets — the player returns to its starting position with
// zero velocity, and the game state returns to Paused.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, current_game_state, enter_game, find_player_sprite, launch_player, run_until_resolved, start_running};
use relativity::{
    game::{
        player::shared::Player,
        shared::types::{Clock, GameItem, Position, Velocity},
    },
    shared::state::GameState,
};
use uom::si::{length::kilometer, velocity::kilometer_per_second};

/// Read the player sprite's position as (x_km, y_km).
fn read_player_position_km(app: &App, entity: Entity) -> (f64, f64) {
    let pos = app.world().get::<Position>(entity).unwrap();
    (pos.x.get::<kilometer>(), pos.y.get::<kilometer>())
}

/// Read the player sprite's velocity as (vx_kms, vy_kms).
fn read_player_velocity_kms(app: &App, entity: Entity) -> (f64, f64) {
    let vel = app.world().get::<Velocity>(entity).unwrap();
    (vel.x.get::<kilometer_per_second>(), vel.y.get::<kilometer_per_second>())
}

/// Count all entities with the `GameItem` component.
fn count_game_items(app: &mut App) -> usize {
    app.world_mut().query_filtered::<Entity, With<GameItem>>().iter(app.world()).count()
}

/// After failure + auto-reset, the level should fully respawn with the player
/// back at the original starting position and zero velocity.
///
/// Steps:
/// 1. Enter game, record starting position and entity count.
/// 2. Launch player toward EARTH to trigger collision → `Failed`.
/// 3. Wait for failure timer to expire (1.5s ≈ 90 frames at 60fps).
/// 4. Verify `GameState::Paused`, player at starting position, zero velocity.
#[test]
fn level_resets_player_to_start_after_failure() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Record the initial starting position.
    let player = find_player_sprite(&mut app);
    let (start_x, start_y) = read_player_position_km(&app, player);
    let initial_item_count = count_game_items(&mut app);

    assert!(initial_item_count > 0, "level should have spawned game items");

    // Launch toward EARTH at (0.28, 0.28) — guaranteed collision.
    launch_player(&mut app, player, (-30_000.0, -30_000.0));
    start_running(&mut app);

    let outcome = run_until_resolved(&mut app, 2000);
    assert_eq!(outcome, GameState::Failed, "player should collide with EARTH");

    // Run frames to let failure timer expire (~1.5s at 60fps = ~90 frames + buffer).
    for _ in 0..100 {
        app.update();
    }

    // Should be back to Paused after the auto-reset.
    assert_eq!(current_game_state(&app), GameState::Paused, "game should auto-reset to Paused after failure");

    // The level should have been respawned — find the new player entity.
    let new_player = find_player_sprite(&mut app);
    let (new_x, new_y) = read_player_position_km(&app, new_player);
    let (new_vx, new_vy) = read_player_velocity_kms(&app, new_player);

    // Player should be back at the starting position.
    let dx = (new_x - start_x).abs();
    let dy = (new_y - start_y).abs();
    assert!(dx < 1.0, "player X should be back at start (delta={dx} km)");
    assert!(dy < 1.0, "player Y should be back at start (delta={dy} km)");

    // Player should have zero velocity (ready for a new launch).
    assert!(new_vx.abs() < 1e-6 && new_vy.abs() < 1e-6, "player velocity should be zero after reset, got ({new_vx}, {new_vy}) km/s");

    // Game items should be fully respawned.
    let new_item_count = count_game_items(&mut app);
    assert_eq!(new_item_count, initial_item_count, "game item count should match after level reset");
}

/// The level reset should also reset the player clock back to zero.
#[test]
fn level_reset_resets_player_clock() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Launch toward EARTH to trigger failure.
    let player = find_player_sprite(&mut app);
    launch_player(&mut app, player, (-30_000.0, -30_000.0));
    start_running(&mut app);

    let outcome = run_until_resolved(&mut app, 2000);
    assert_eq!(outcome, GameState::Failed);

    // Wait for failure timer to expire.
    for _ in 0..100 {
        app.update();
    }

    assert_eq!(current_game_state(&app), GameState::Paused);

    // Find the new player clock entity and verify it's at zero.
    let clock_entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Player>, With<Clock>)>()
        .single(app.world())
        .expect("expected exactly one Player clock entity after reset");

    let clock_val = app.world().get::<Clock>(clock_entity).unwrap();
    let seconds = clock_val.value.get::<uom::si::time::second>();
    assert!(seconds.abs() < 1e-6, "player clock should be zero after reset, got {seconds} s");
}
