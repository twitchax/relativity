// E2E headless test: verifies that the full gameplay loop works for Level 1.
//
// The level is spawned exactly as the real game does — the player starts at
// the default position with zero velocity.  We then set a launch velocity
// (mimicking what the launch systems compute from mouse input) and run the
// full physics loop to prove the level is solvable.
//
// UPDATE THE VELOCITIES below if you change the Level 1 layout.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use common::{build_gameplay_app, enter_game, find_player_clock, find_player_sprite, launch_player, read_player_clock_days, run_until_resolved, start_running};
use relativity::shared::state::GameState;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Verify the full success path: player reaches the destination and the game
/// transitions to `GameState::Finished`.
///
/// Level 1 layout (percentage coords):
///   Player:      (0.30, 0.30)  — default spawn, zero velocity
///   EARTH:       (0.28, 0.28)  — near player, low mass
///   SUN:         (0.50, 0.50)  — central, blocks direct diagonal to destination
///   SUN2:        (0.80, 0.70)  — secondary star
///   Destination: (0.90, 0.90)  — large radius (4× UNIT_RADIUS), mass 0.6× SUN
///
/// The launch velocity is aimed above the SUN to arc over it.
/// Gravity bends the trajectory back down toward the destination.
/// This proves the level is solvable from the default starting position.
#[test]
fn level1_is_solvable() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);
    let player_clock = find_player_clock(&mut app);

    // Launch toward the upper-right, angled above the SUN at (0.5, 0.5).
    // From (0.3, 0.3), gravity from the SUN curves the trajectory back
    // downward toward the destination at (0.9, 0.9).
    launch_player(&mut app, player, (100_000.0, 200_000.0));

    start_running(&mut app);

    let outcome = run_until_resolved(&mut app, 2000);

    assert_eq!(outcome, GameState::Finished, "Level 1 should be solvable — expected GameState::Finished, got {outcome:?}");

    let player_time = read_player_clock_days(&app, player_clock);
    assert!(player_time < 0.20, "player clock should be less than 0.20 days, got {player_time}");
}

/// Verify the failure path: launching toward a nearby planet causes a
/// collision and the game transitions to `GameState::Failed`.
///
/// The player starts at (0.30, 0.30) and EARTH is at (0.28, 0.28) — very
/// close.  A slow velocity aimed toward EARTH guarantees a collision.
#[test]
fn level1_direct_shot_at_earth_collides() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);

    // Launch slowly toward EARTH at (0.28, 0.28).
    // Player is at (0.3, 0.3), so direction is (-1, -1) normalized.
    // EARTH has radius 2× UNIT_RADIUS ≈ 120M km — hard to miss at this range.
    launch_player(&mut app, player, (-30_000.0, -30_000.0));

    start_running(&mut app);

    let outcome = run_until_resolved(&mut app, 2000);

    assert_eq!(outcome, GameState::Failed, "expected GameState::Failed (player should collide with EARTH), got {outcome:?}");
}
