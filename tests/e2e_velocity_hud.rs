// E2E headless test: verifies the player HUD displays velocity as a fraction
// of c (e.g., "v = 0.42c") alongside the clock and gamma values.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game, find_player_sprite, launch_player, start_running};
use relativity::game::shared::types::PlayerHud;

/// After launching at a known velocity and running a few frames, the HUD text
/// entity (marked with `PlayerHud`) should contain a velocity string like
/// "v = 0.71c" alongside clock and gamma values.
#[test]
fn hud_displays_velocity_fraction_of_c() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);

    // Launch at ~0.71c (each component ~150_000 km/s → scalar ≈ 212_132 km/s ≈ 0.71c).
    launch_player(&mut app, player, (150_000.0, 150_000.0));
    start_running(&mut app);

    // Run several frames so player_clock_update and player_clock_text_update execute.
    for _ in 0..5 {
        app.update();
    }

    // Query the HUD text entity.
    let hud_text = app
        .world_mut()
        .query_filtered::<&Text, With<PlayerHud>>()
        .single(app.world())
        .expect("expected a PlayerHud text entity");

    let text_str: &str = hud_text;

    // The HUD should contain all four readouts.
    assert!(text_str.contains("t_p ="), "HUD should display player clock (t_p), got: {text_str}");
    assert!(text_str.contains("γ_v ="), "HUD should display velocity gamma (γ_v), got: {text_str}");
    assert!(text_str.contains("γ_g ="), "HUD should display gravitational gamma (γ_g), got: {text_str}");
    assert!(text_str.contains("v ="), "HUD should display velocity fraction (v = …c), got: {text_str}");

    // Verify the velocity fraction format: "v = X.XXc" with a reasonable value.
    // At ~0.71c we expect something like "v = 0.71c" (exact value depends on gravity).
    // Use rfind to skip the "γ_v = " occurrence and find the standalone "v = ".
    let vel_idx = text_str.rfind("v = ").expect("could not find 'v = ' in HUD text");
    let vel_part = &text_str[vel_idx + 4..]; // skip "v = "
    assert!(vel_part.contains('c'), "velocity display should end with 'c', got: {vel_part}");

    // Parse the numeric portion to verify it's a reasonable fraction of c.
    let numeric: f64 = vel_part.trim_end_matches('c').trim().parse().expect("could not parse velocity fraction");
    assert!(numeric > 0.0, "velocity fraction should be positive after launch, got: {numeric}");
    assert!(numeric < 1.0, "velocity fraction should be < 1.0 (sub-luminal), got: {numeric}");
}

/// At rest (before launch), the HUD should show v = 0.00c.
#[test]
fn hud_displays_zero_velocity_at_rest() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Don't launch — player stays at rest. Transition to Running so the
    // text update system executes.
    start_running(&mut app);

    for _ in 0..3 {
        app.update();
    }

    let hud_text = app
        .world_mut()
        .query_filtered::<&Text, With<PlayerHud>>()
        .single(app.world())
        .expect("expected a PlayerHud text entity");

    let text_str: &str = hud_text;

    assert!(text_str.contains("v = 0.00c"), "HUD should show 'v = 0.00c' at rest, got: {text_str}");
}
