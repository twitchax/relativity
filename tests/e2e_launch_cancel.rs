// E2E headless test: verifies launch cancel behavior via right-click and Escape.
//
// `launch_cancel_system` resets LaunchState to Idle from any non-Idle state
// when the player right-clicks or presses Escape.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::{player::player_sprite::launch_cancel_system, shared::types::LaunchState};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn current_launch_state(app: &App) -> LaunchState {
    app.world().resource::<LaunchState>().clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Right-click during AimLocked resets to Idle.
#[test]
fn cancel_aim_locked_via_right_click() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: 1.0 };

    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
    app.world_mut().run_system_once(launch_cancel_system).expect("launch_cancel_system should run");

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}

/// Escape during AimLocked resets to Idle.
#[test]
fn cancel_aim_locked_via_escape() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: 0.5 };

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
    app.world_mut().run_system_once(launch_cancel_system).expect("launch_cancel_system should run");

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}

/// Right-click during Launching resets to Idle.
#[test]
fn cancel_launching_via_right_click() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 0.0, power: 0.5 };

    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
    app.world_mut().run_system_once(launch_cancel_system).expect("launch_cancel_system should run");

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}

/// Escape during Launching resets to Idle.
#[test]
fn cancel_launching_via_escape() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 1.0, power: 0.8 };

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
    app.world_mut().run_system_once(launch_cancel_system).expect("launch_cancel_system should run");

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}

/// Cancel is a no-op when already Idle.
#[test]
fn cancel_noop_when_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    assert_eq!(current_launch_state(&app), LaunchState::Idle);

    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Right);
    app.world_mut().run_system_once(launch_cancel_system).expect("launch_cancel_system should run");

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}
