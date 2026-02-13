// E2E headless test: verifies that the HUD re-spawns correctly after a level
// reset triggered by PendingLevelReset (player collision → failure → auto-reset).
//
// Validates acceptance criterion uat-007: "HUD renders correctly after level
// reset (PendingLevelReset re-spawn)".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, current_game_state, enter_game, find_player_sprite, launch_player, run_until_resolved, start_running};
use relativity::{
    game::hud::{HudGravGamma, HudObserverTime, HudPlayerTime, HudSimRate, HudVelocityFraction, HudVelocityGamma},
    shared::state::GameState,
};

/// After failure + auto-reset, all six HUD readout entities must exist and
/// display their default values (same as a fresh level spawn).
#[test]
fn hud_respawns_with_all_readouts_after_level_reset() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Sanity: HUD exists before reset.
    let pre_tp = app.world_mut().query_filtered::<Entity, With<HudPlayerTime>>().iter(app.world()).count();
    assert_eq!(pre_tp, 1, "HudPlayerTime should exist before reset");

    // Trigger failure: launch toward EARTH to collide.
    let player = find_player_sprite(&mut app);
    launch_player(&mut app, player, (-30_000.0, -30_000.0));
    start_running(&mut app);

    let outcome = run_until_resolved(&mut app, 2000);
    assert_eq!(outcome, GameState::Failed, "player should collide with EARTH");

    // Wait for failure timer to expire and level to reset (~1.5s at 60fps).
    for _ in 0..100 {
        app.update();
    }

    assert_eq!(current_game_state(&app), GameState::Paused, "game should auto-reset to Paused");

    // Verify all six HUD readout entities are present after reset.
    let tp_count = app.world_mut().query_filtered::<Entity, With<HudPlayerTime>>().iter(app.world()).count();
    assert_eq!(tp_count, 1, "expected exactly one HudPlayerTime after reset");

    let vg_count = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().iter(app.world()).count();
    assert_eq!(vg_count, 1, "expected exactly one HudVelocityGamma after reset");

    let gg_count = app.world_mut().query_filtered::<Entity, With<HudGravGamma>>().iter(app.world()).count();
    assert_eq!(gg_count, 1, "expected exactly one HudGravGamma after reset");

    let vf_count = app.world_mut().query_filtered::<Entity, With<HudVelocityFraction>>().iter(app.world()).count();
    assert_eq!(vf_count, 1, "expected exactly one HudVelocityFraction after reset");

    let to_count = app.world_mut().query_filtered::<Entity, With<HudObserverTime>>().iter(app.world()).count();
    assert_eq!(to_count, 1, "expected exactly one HudObserverTime after reset");

    let sr_count = app.world_mut().query_filtered::<Entity, With<HudSimRate>>().iter(app.world()).count();
    assert_eq!(sr_count, 1, "expected exactly one HudSimRate after reset");

    // Verify readouts show default values (level-fresh state).
    let tp_text = app.world_mut().query_filtered::<&Text2d, With<HudPlayerTime>>().single(app.world()).unwrap().0.clone();
    assert_eq!(tp_text, "t_p = 0.00", "t_p should reset to default: {tp_text}");

    let vg_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityGamma>>().single(app.world()).unwrap().0.clone();
    assert_eq!(vg_text, "γ_v = 1.00", "γ_v should reset to default: {vg_text}");

    let gg_text = app.world_mut().query_filtered::<&Text2d, With<HudGravGamma>>().single(app.world()).unwrap().0.clone();
    assert_eq!(gg_text, "γ_g = 1.00", "γ_g should reset to default: {gg_text}");

    let vf_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityFraction>>().single(app.world()).unwrap().0.clone();
    assert_eq!(vf_text, "v = 0.00c", "v should reset to default: {vf_text}");
}
