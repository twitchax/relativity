// E2E headless test: verifies that a numeric velocity readout (e.g. "0.45c")
// is spawned near the radial arc during the Launching phase and is despawned
// when the launch state returns to Idle.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::{
    player::player_sprite::{launch_readout_system, map_power_nonlinear},
    shared::types::{LaunchState, VelocityReadout},
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// During the Launching phase the readout system spawns a `VelocityReadout`
/// entity whose `Text2d` content matches the expected "X.XXc" format.
#[test]
fn readout_spawns_with_correct_text_during_launching() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let power = 0.6_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching {
        angle: std::f32::consts::FRAC_PI_4,
        power,
    };

    app.world_mut().run_system_once(launch_readout_system).expect("launch_readout_system should run");
    app.update();

    // Query for the spawned VelocityReadout entity.
    let mut query = app.world_mut().query_filtered::<&Text2d, With<VelocityReadout>>();
    let texts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(texts.len(), 1, "expected exactly one VelocityReadout entity");

    let expected_fraction = map_power_nonlinear(power) * 0.99;
    let expected_text = format!("{expected_fraction:.2}c");
    assert_eq!(**texts[0], expected_text, "readout text should match velocity fraction");
}

/// When the launch state returns to Idle the readout entity is despawned.
#[test]
fn readout_despawns_when_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // First, enter Launching to spawn the readout.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 0.0, power: 0.5 };
    app.world_mut().run_system_once(launch_readout_system).expect("launch_readout_system should run");
    app.update();

    // Verify readout exists.
    let count = app.world_mut().query_filtered::<Entity, With<VelocityReadout>>().iter(app.world()).count();
    assert_eq!(count, 1, "readout should exist after Launching");

    // Return to Idle.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Idle;
    app.world_mut().run_system_once(launch_readout_system).expect("launch_readout_system should run in Idle");
    app.update();

    let count = app.world_mut().query_filtered::<Entity, With<VelocityReadout>>().iter(app.world()).count();
    assert_eq!(count, 0, "readout should be despawned after returning to Idle");
}

/// The readout text updates when power changes between frames.
#[test]
fn readout_updates_on_power_change() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Spawn at low power.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 0.0, power: 0.2 };
    app.world_mut().run_system_once(launch_readout_system).expect("launch_readout_system should run");
    app.update();

    // Increase power.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 0.0, power: 0.9 };
    app.world_mut().run_system_once(launch_readout_system).expect("launch_readout_system should run at higher power");
    app.update();

    let mut query = app.world_mut().query_filtered::<&Text2d, With<VelocityReadout>>();
    let texts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(texts.len(), 1, "should still be exactly one readout");

    let expected_fraction = map_power_nonlinear(0.9) * 0.99;
    let expected_text = format!("{expected_fraction:.2}c");
    assert_eq!(**texts[0], expected_text, "readout text should reflect updated power");
}
