// E2E headless test: verifies that HUD value changes trigger a subtle visual
// feedback via the HudFlash component (brightness boost that decays over time).
//
// This validates acceptance criterion uat-005: "Value changes trigger a subtle
// visual feedback (color flash, glow pulse, or highlight)".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use bevy_lunex::prelude::*;
use common::{build_gameplay_app, enter_game, find_player_sprite, launch_player, start_running};
use relativity::game::hud::{HudFlash, HudGravGamma, HudObserverTime, HudPlayerTime, HudSimRate, HudVelocityFraction, HudVelocityGamma};

/// Every numeric readout entity must carry a `HudFlash` component so the
/// flash system can detect text changes and apply a brightness boost.
#[test]
fn all_value_readouts_have_hud_flash_component() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let count = |world: &mut World, name: &str, entity: Entity| {
        assert!(world.get::<HudFlash>(entity).is_some(), "{name} should have HudFlash component");
    };

    let tp = app.world_mut().query_filtered::<Entity, With<HudPlayerTime>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudPlayerTime", tp);

    let vg = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudVelocityGamma", vg);

    let gg = app.world_mut().query_filtered::<Entity, With<HudGravGamma>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudGravGamma", gg);

    let vf = app.world_mut().query_filtered::<Entity, With<HudVelocityFraction>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudVelocityFraction", vf);

    let to = app.world_mut().query_filtered::<Entity, With<HudObserverTime>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudObserverTime", to);

    let sr = app.world_mut().query_filtered::<Entity, With<HudSimRate>>().single(app.world()).unwrap();
    count(app.world_mut(), "HudSimRate", sr);
}

/// When the player launches and HUD text changes, the flash system should
/// briefly boost the readout color brightness. After the flash decays, the
/// color should return to the base value (lower brightness).
#[test]
fn value_change_triggers_brightness_flash() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player = find_player_sprite(&mut app);

    // Record baseline color of velocity-gamma readout (γ_v). At spawn the
    // flash timer is finished, so the color should be the base TEXT_COLOR.
    let baseline_color = {
        let entity = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().single(app.world()).unwrap();
        app.world().get::<UiColor>(entity).unwrap().get(&UiBase::id()).unwrap().to_srgba()
    };

    // Launch at ~0.71c and start the physics simulation.
    launch_player(&mut app, player, (150_000.0, 150_000.0));
    start_running(&mut app);

    // Run 2 frames: text updates trigger gamma color + flash boost.
    app.update();
    app.update();

    let during_flash = {
        let entity = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().single(app.world()).unwrap();
        app.world().get::<UiColor>(entity).unwrap().get(&UiBase::id()).unwrap().to_srgba()
    };

    // Run 20 more frames so flash fully decays (20 × 1/60 ≈ 0.33s > 0.25s).
    for _ in 0..20 {
        app.update();
    }

    let after_flash = {
        let entity = app.world_mut().query_filtered::<Entity, With<HudVelocityGamma>>().single(app.world()).unwrap();
        app.world().get::<UiColor>(entity).unwrap().get(&UiBase::id()).unwrap().to_srgba()
    };

    // During the flash, at least one RGB channel should be boosted above the
    // post-flash (settled) color. The flash adds up to FLASH_BOOST (0.35) to
    // each channel.
    let boosted = during_flash.red > after_flash.red + 0.01 || during_flash.green > after_flash.green + 0.01 || during_flash.blue > after_flash.blue + 0.01;

    assert!(
        boosted,
        "expected brightness boost during flash.\n  during: ({:.3}, {:.3}, {:.3})\n  after:  ({:.3}, {:.3}, {:.3})\n  base:   ({:.3}, {:.3}, {:.3})",
        during_flash.red, during_flash.green, during_flash.blue, after_flash.red, after_flash.green, after_flash.blue, baseline_color.red, baseline_color.green, baseline_color.blue,
    );
}
