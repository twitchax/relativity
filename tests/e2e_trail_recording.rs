// E2E headless test: verifies that the trail system records positions behind
// the player, colored by total gamma (velocity × gravitational).

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use common::{build_gameplay_app, enter_game, find_player_clock, find_player_sprite, launch_player, start_running};
use relativity::game::shared::types::{GravitationalGamma, TrailBuffer, VelocityGamma};

/// After launching the player and running physics for several frames, the
/// `TrailBuffer` should contain recorded positions with gamma-derived colors.
#[test]
fn trail_records_positions_colored_by_gamma() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);
    let player_clock = find_player_clock(&mut app);

    // Launch the player so physics actually runs.
    launch_player(&mut app, player, (150_000.0, 150_000.0));
    start_running(&mut app);

    // Run 60 frames (1 second of game time).
    for _ in 0..60 {
        app.update();
    }

    // Read TrailBuffer from the player entity.
    let world = app.world();

    let trail = world.get::<TrailBuffer>(player).expect("player should have TrailBuffer component");

    // After 60 frames we expect a non-trivial number of trail points.
    assert!(trail.points.len() >= 30, "expected at least 30 trail points after 60 frames, got {}", trail.points.len());

    // Verify that trail point colors correspond to the gamma values.
    // Gamma components live on the player clock entity.
    let vel_gamma = world.get::<VelocityGamma>(player_clock).expect("player clock should have VelocityGamma");
    let grav_gamma = world.get::<GravitationalGamma>(player_clock).expect("player clock should have GravitationalGamma");
    let total_gamma = vel_gamma.value * grav_gamma.value;

    // The most recent trail point should have a color consistent with gamma_to_color logic:
    // γ ≈ 1 → cool (low red, high blue), γ > 2 → warm (high red, low blue).
    let (_last_pos, last_color) = trail.points.last().expect("trail should have at least one point");
    let srgba = last_color.to_srgba();

    // All channels must be valid sRGBA in [0, 1].
    assert!((0.0..=1.0).contains(&srgba.red), "red out of range: {}", srgba.red);
    assert!((0.0..=1.0).contains(&srgba.green), "green out of range: {}", srgba.green);
    assert!((0.0..=1.0).contains(&srgba.blue), "blue out of range: {}", srgba.blue);
    assert!((0.0..=1.0).contains(&srgba.alpha), "alpha out of range: {}", srgba.alpha);

    // Verify color changes with gamma: replicate the mapping formula inline.
    // gamma_to_color: blend = ((gamma - 1) / 2).clamp(0, 1)
    //   red = 0.2 + blend * 0.8, blue = 1.0 - blend
    let blend = ((total_gamma - 1.0) / 2.0).clamp(0.0, 1.0) as f32;
    let expected_red = 0.2 + blend * 0.8;
    let expected_blue = 1.0 - blend;
    let eps = 0.01;
    assert!(
        (srgba.red - expected_red).abs() < eps && (srgba.blue - expected_blue).abs() < eps,
        "trail color should match gamma mapping (γ={total_gamma:.3}): expected r={expected_red:.3} b={expected_blue:.3}, got r={:.3} b={:.3}",
        srgba.red,
        srgba.blue,
    );

    // Verify that positions are distinct (player is moving, so trail points differ).
    let first_pos = trail.points[0].0;
    let mid_pos = trail.points[trail.points.len() / 2].0;
    assert_ne!(first_pos, mid_pos, "trail should contain distinct positions as the player moves");
}
