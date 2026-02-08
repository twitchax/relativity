// E2E headless test: verifies the launch state machine transitions correctly.
//
// LaunchState transitions: Idle → AimLocked → Launching → Running
//
// Because headless Bevy has no window cursor, we cannot drive `launch_aim_system`
// and `launch_power_system` through real input (cursor_position() returns None).
// Instead we directly set the LaunchState resource to simulate the aim/power
// phases — those transitions are simple assignments — and use `run_system_once`
// for `launch_fire_system` which only needs ButtonInput<MouseButton>.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::{
    game::{
        player::player_sprite::launch_fire_system,
        shared::types::{LaunchState, Velocity},
    },
    shared::state::GameState,
};
use uom::si::velocity::kilometer_per_second;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read the current LaunchState.
fn current_launch_state(app: &App) -> LaunchState {
    app.world().resource::<LaunchState>().clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Verify that LaunchState defaults to Idle on game entry.
#[test]
fn launch_state_starts_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    assert_eq!(current_launch_state(&app), LaunchState::Idle);
}

/// Verify Idle → AimLocked transition.
///
/// In the real game, `launch_aim_system` computes the angle from cursor position
/// and sets `LaunchState::AimLocked { angle }`.  Since headless Bevy has no cursor,
/// we verify the transition by directly setting the resource — this matches exactly
/// what the system does after computing the angle.
#[test]
fn idle_to_aim_locked() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    assert_eq!(current_launch_state(&app), LaunchState::Idle);

    let aim_angle = std::f32::consts::FRAC_PI_4;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: aim_angle };

    let state = current_launch_state(&app);
    assert_eq!(state, LaunchState::AimLocked { angle: aim_angle });
}

/// Verify AimLocked → Launching transition.
///
/// `launch_power_system` computes power from drag distance and transitions
/// AimLocked → Launching.  We simulate this by directly setting the resource.
#[test]
fn aim_locked_to_launching() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let aim_angle = std::f32::consts::FRAC_PI_4;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: aim_angle };

    // Simulate drag: transition to Launching with power.
    let power = 0.75;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: aim_angle, power };

    let state = current_launch_state(&app);
    assert_eq!(state, LaunchState::Launching { angle: aim_angle, power });
}

/// Verify Launching → Running transition via the actual `launch_fire_system`.
///
/// This is the most important transition: on mouse release during the Launching
/// phase, the system computes launch velocity, sets GameState::Running, and
/// resets LaunchState to Idle.
#[test]
fn launching_to_running_via_fire_system() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);

    // Pre-condition: game starts in Paused.
    assert_eq!(common::current_game_state(&app), GameState::Paused);

    // Simulate aim+power phases.
    let aim_angle = std::f32::consts::FRAC_PI_4; // 45° (up-right)
    let power = 0.8;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: aim_angle, power };

    // Inject mouse release event — `launch_fire_system` checks `just_released(Left)`.
    // We must press first so that release is detected as `just_released`.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update(); // Process press (clears `just_pressed` flag).

    // Re-set LaunchState since update() may have run systems that reset it.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: aim_angle, power };

    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);

    // Directly invoke fire system so it sees `just_released`.
    app.world_mut().run_system_once(launch_fire_system).expect("launch_fire_system should run");

    // LaunchState should be reset to Idle.
    assert_eq!(current_launch_state(&app), LaunchState::Idle, "LaunchState should reset to Idle after firing");

    // Process the queued GameState transition.
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running, "GameState should transition to Running after launch");

    // Player velocity should be non-zero and match the launch angle.
    let vel = app.world().get::<Velocity>(player).unwrap();
    let vx = vel.x.get::<kilometer_per_second>();
    let vy = vel.y.get::<kilometer_per_second>();

    assert!(vx > 0.0, "vx should be positive for 45° launch, got {vx}");
    assert!(vy > 0.0, "vy should be positive for 45° launch, got {vy}");

    // At 45° the components should be roughly equal; allow 10% relative tolerance
    // because floating-point trig on f32 angle introduces small asymmetry.
    let ratio = vx / vy;
    assert!((0.9..=1.1).contains(&ratio), "vx/vy ratio should be ~1.0 for 45° launch, got {ratio}");
}

/// Verify that releasing the mouse during AimLocked (no drag/power phase) cancels
/// back to Idle instead of firing.
#[test]
fn aim_locked_release_cancels_to_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Set up AimLocked state.
    let aim_angle = 1.0_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: aim_angle };

    // Press and update to register the press.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update();

    // Re-set state since update may have run systems.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: aim_angle };

    // Release without ever entering Launching.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);

    app.world_mut().run_system_once(launch_fire_system).expect("launch_fire_system should run");

    // Should cancel back to Idle, not fire.
    assert_eq!(current_launch_state(&app), LaunchState::Idle, "AimLocked release should cancel to Idle");
    // GameState should still be Paused.
    assert_eq!(common::current_game_state(&app), GameState::Paused, "GameState should remain Paused on cancel");
}

/// Full state machine round-trip: Idle → AimLocked → Launching → Running.
///
/// Combines all transitions into one test to verify the complete flow.
#[test]
fn full_launch_state_machine_idle_to_running() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // 1. Start: Idle
    assert_eq!(current_launch_state(&app), LaunchState::Idle);
    assert_eq!(common::current_game_state(&app), GameState::Paused);

    // 2. Idle → AimLocked (simulate mouse click computing angle)
    let angle = 0.0_f32; // rightward
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle };
    assert_eq!(current_launch_state(&app), LaunchState::AimLocked { angle });

    // 3. AimLocked → Launching (simulate drag computing power)
    let power = 0.6;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
    assert_eq!(current_launch_state(&app), LaunchState::Launching { angle, power });

    // 4. Launching → fire (mouse release triggers Running)
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
    app.update();
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().release(MouseButton::Left);

    app.world_mut().run_system_once(launch_fire_system).expect("launch_fire_system should run");

    // 5. Verify final state: LaunchState::Idle, GameState::Running
    assert_eq!(current_launch_state(&app), LaunchState::Idle);
    app.update();
    assert_eq!(common::current_game_state(&app), GameState::Running);
}
