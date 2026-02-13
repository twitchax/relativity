// E2E headless test: verifies that HUD readout colors shift from cool
// (blue/cyan) toward warm (orange/red) as γ increases.
//
// This validates acceptance criterion uat-006: "Gamma-based color shifting:
// readouts shift from cool (blue/cyan) toward warm (orange/red) as γ increases".
//
// NOTE: `player_hud_text_update` only runs during `GameState::Running`, and the
// `hud_flash_system` captures the current UiColor as its base on text change.
// The gamma color becomes visible after a text change (launch) + flash decay.
// At spawn the text is "γ_v = 1.00" and gamma defaults to 1.0, so the text
// doesn't change until the player gains noticeable velocity.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use bevy_lunex::prelude::*;
use common::{build_gameplay_app, enter_game, find_player_sprite, launch_player, start_running};
use relativity::game::hud::{HudVelocityFraction, HudVelocityGamma};

/// Helper: extract the settled SRGBA color from a HUD readout entity.
fn read_ui_color(app: &App, entity: Entity) -> Srgba {
    app.world().get::<UiColor>(entity).unwrap().get(&UiBase::id()).unwrap().to_srgba()
}

/// Helper: build app, enter game, launch at given velocity, run for enough
/// frames (0.67s) for physics + HUD text update + flash decay, then return
/// the settled color of the queried readout entity.
fn settled_color_after_launch<T: Component>(vel_kms: (f64, f64)) -> Srgba {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);
    launch_player(&mut app, player, vel_kms);
    start_running(&mut app);

    // 40 frames at 1/60s ≈ 0.67s — well past the 0.25s flash duration.
    for _ in 0..40 {
        app.update();
    }

    let entity = app.world_mut().query_filtered::<Entity, With<T>>().single(app.world()).unwrap();
    read_ui_color(&app, entity)
}

/// At modest speed (γ ≈ 1.01), the velocity-gamma readout color should be
/// cool (blue > red), matching the nominal cyan anchor.
#[test]
fn low_gamma_readout_is_cool() {
    // ~42,400 km/s ≈ 0.14c → γ ≈ 1.01, text changes to "γ_v = 1.01".
    let color = settled_color_after_launch::<HudVelocityGamma>((30_000.0, 30_000.0));

    assert!(
        color.blue > color.red,
        "at low γ expected cool color (blue > red), got r={:.3} g={:.3} b={:.3}",
        color.red,
        color.green,
        color.blue,
    );
}

/// At very high speed (γ ≥ 3), the velocity-gamma readout color should be
/// warm (red > blue), matching the extreme red/orange anchor.
#[test]
fn high_gamma_readout_is_warm() {
    // ~283,000 km/s ≈ 0.94c → γ ≈ 3, text changes to "γ_v = 3.xx".
    let color = settled_color_after_launch::<HudVelocityGamma>((200_000.0, 200_000.0));

    assert!(
        color.red > color.blue,
        "at high γ expected warm color (red > blue), got r={:.3} g={:.3} b={:.3}",
        color.red,
        color.green,
        color.blue,
    );
}

/// The velocity-fraction readout also uses gamma-based coloring and should
/// shift warm at high speed (it tracks velocity gamma, not gravitational).
#[test]
fn velocity_fraction_readout_shifts_warm_at_high_speed() {
    let color = settled_color_after_launch::<HudVelocityFraction>((200_000.0, 200_000.0));

    assert!(
        color.red > color.blue,
        "velocity fraction at high γ should be warm (red > blue), got r={:.3} b={:.3}",
        color.red,
        color.blue,
    );
}
