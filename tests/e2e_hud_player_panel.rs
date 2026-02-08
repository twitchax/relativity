// E2E headless test: verifies that the player panel spawns four HUD labels
// (t_p, γ_v, γ_g, v) with correct initial and updated values.
//
// This validates acceptance criterion uat-002: "Player panel shows t_p, γ_v,
// γ_g, v with correct values".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game, find_player_sprite, launch_player, start_running};
use relativity::game::hud::{HudGravGamma, HudPlayerTime, HudVelocityFraction, HudVelocityGamma};

/// After entering InGame, the player panel must contain exactly one entity
/// for each of the four player stats: t_p, γ_v, γ_g, v.
#[test]
fn player_panel_has_four_stat_labels() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let tp_count = app.world_mut().query_filtered::<Entity, With<HudPlayerTime>>().iter(app.world()).count();
    assert_eq!(tp_count, 1, "expected exactly one HudPlayerTime label");

    let vg_count = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().iter(app.world()).count();
    assert_eq!(vg_count, 1, "expected exactly one HudVelocityGamma label");

    let gg_count = app.world_mut().query_filtered::<Entity, With<HudGravGamma>>().iter(app.world()).count();
    assert_eq!(gg_count, 1, "expected exactly one HudGravGamma label");

    let vf_count = app.world_mut().query_filtered::<Entity, With<HudVelocityFraction>>().iter(app.world()).count();
    assert_eq!(vf_count, 1, "expected exactly one HudVelocityFraction label");
}

/// At spawn, the four player stat labels should show default values
/// (t_p = 0.00, γ_v = 1.00, γ_g = 1.00, v = 0.00c).
#[test]
fn player_panel_shows_default_values_at_spawn() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let tp_text = app.world_mut().query_filtered::<&Text2d, With<HudPlayerTime>>().single(app.world()).unwrap().0.clone();
    assert_eq!(tp_text, "t_p = 0.00", "initial t_p text: {tp_text}");

    let vg_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityGamma>>().single(app.world()).unwrap().0.clone();
    assert_eq!(vg_text, "γ_v = 1.00", "initial γ_v text: {vg_text}");

    let gg_text = app.world_mut().query_filtered::<&Text2d, With<HudGravGamma>>().single(app.world()).unwrap().0.clone();
    assert_eq!(gg_text, "γ_g = 1.00", "initial γ_g text: {gg_text}");

    let vf_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityFraction>>().single(app.world()).unwrap().0.clone();
    assert_eq!(vf_text, "v = 0.00c", "initial v text: {vf_text}");
}

/// After launching at a known velocity and running several frames, the
/// player HUD labels must reflect updated values: γ_v > 1, t_p > 0,
/// and v showing a non-zero fraction of c.
#[test]
fn player_panel_updates_with_correct_values_after_launch() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);

    // Launch at ~0.71c (each component ~150_000 km/s).
    launch_player(&mut app, player, (150_000.0, 150_000.0));
    start_running(&mut app);

    // Run several frames so player_clock_update and player_hud_text_update execute.
    for _ in 0..10 {
        app.update();
    }

    // t_p should have advanced beyond 0.
    let tp_text = app.world_mut().query_filtered::<&Text2d, With<HudPlayerTime>>().single(app.world()).unwrap().0.clone();
    assert!(tp_text.starts_with("t_p = "), "t_p label format wrong: {tp_text}");
    assert_ne!(tp_text, "t_p = 0.00", "t_p should have advanced: {tp_text}");

    // γ_v should be > 1 at ~0.71c.
    let vg_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityGamma>>().single(app.world()).unwrap().0.clone();
    assert!(vg_text.starts_with("γ_v = "), "γ_v label format wrong: {vg_text}");
    let vg_value: f64 = vg_text.trim_start_matches("γ_v = ").parse().unwrap();
    assert!(vg_value > 1.0, "γ_v should be > 1 at ~0.71c, got: {vg_value}");

    // γ_g should be >= 1 (may be slightly above 1 due to planet gravity).
    let gg_text = app.world_mut().query_filtered::<&Text2d, With<HudGravGamma>>().single(app.world()).unwrap().0.clone();
    assert!(gg_text.starts_with("γ_g = "), "γ_g label format wrong: {gg_text}");
    let gg_value: f64 = gg_text.trim_start_matches("γ_g = ").parse().unwrap();
    assert!(gg_value >= 1.0, "γ_g should be >= 1, got: {gg_value}");

    // v should show a non-zero fraction of c.
    let vf_text = app.world_mut().query_filtered::<&Text2d, With<HudVelocityFraction>>().single(app.world()).unwrap().0.clone();
    assert!(vf_text.starts_with("v = "), "v label format wrong: {vf_text}");
    assert!(vf_text.ends_with('c'), "v label should end with 'c': {vf_text}");
    let v_value: f64 = vf_text.trim_start_matches("v = ").trim_end_matches('c').parse().unwrap();
    assert!(v_value > 0.0, "v should be > 0 after launch, got: {v_value}");
    assert!(v_value < 1.0, "v should be < 1 (sub-luminal), got: {v_value}");
}
