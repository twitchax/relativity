// E2E headless test: verifies that the observer panel spawns a HUD label
// (t_o) with correct initial and updated values.
//
// This validates acceptance criterion uat-003: "Observer panel shows t_o
// with correct values".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game, start_running};
use relativity::game::hud::HudObserverTime;

/// After entering InGame, the observer panel must contain exactly one
/// `HudObserverTime` entity.
#[test]
fn observer_panel_has_time_label() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let count = app.world_mut().query_filtered::<Entity, With<HudObserverTime>>().iter(app.world()).count();
    assert_eq!(count, 1, "expected exactly one HudObserverTime label");
}

/// At spawn, the observer time label should show the default value `t_o = 0.00`.
#[test]
fn observer_panel_shows_default_value_at_spawn() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let text = app.world_mut().query_filtered::<&Text2d, With<HudObserverTime>>().single(app.world()).unwrap().0.clone();
    assert_eq!(text, "t_o = 0.00", "initial t_o text: {text}");
}

/// After running several frames, the observer clock should advance and the
/// HUD label should reflect a value greater than zero, formatted as
/// `t_o = X.XX`.
#[test]
fn observer_panel_updates_with_correct_values_after_running() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    start_running(&mut app);

    // Run several frames so observer_clock_update and observer_hud_text_update execute.
    for _ in 0..10 {
        app.update();
    }

    let text = app.world_mut().query_filtered::<&Text2d, With<HudObserverTime>>().single(app.world()).unwrap().0.clone();

    assert!(text.starts_with("t_o = "), "t_o label format wrong: {text}");
    assert_ne!(text, "t_o = 0.00", "t_o should have advanced: {text}");

    // Parse the numeric value and verify it is positive.
    let value: f64 = text.trim_start_matches("t_o = ").parse().unwrap();
    assert!(value > 0.0, "t_o should be > 0 after running, got: {value}");
}
