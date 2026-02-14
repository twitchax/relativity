// E2E headless test: verifies that launch visuals (direction gizmo line and
// radial power arc) render correctly during the AimLocked and Launching phases.
//
// Both the direction line and the radial arc are drawn via Bevy Gizmos
// (immediate-mode GPU drawing) and cannot be queried as ECS entities.  We verify
// them by running the `launch_visual_system` in each state and confirming it
// completes without panicking — the gizmo draw calls execute on the correct
// code path.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::ecs::system::RunSystemOnce;
use relativity::game::{player::player_sprite::launch_visual_system, shared::types::LaunchState};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// In AimLocked state, the direction gizmo line and faint arc outline are drawn
/// without panicking.
#[test]
fn aim_locked_draws_direction_line_and_arc_outline() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Set AimLocked with a 45° angle.
    let angle = std::f32::consts::FRAC_PI_4;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle };

    // Run the visual system — gizmo line_2d and arc_2d are called inside.
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in AimLocked state");
    app.update();
}

/// In Launching state, the direction line and filled radial arc are drawn via
/// Gizmos without panicking.
#[test]
fn launching_draws_direction_line_and_filled_arc() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = std::f32::consts::FRAC_PI_4;
    let power = 0.6_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };

    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in Launching state");
    app.update();
}

/// When LaunchState returns to Idle, the visual system completes without error.
#[test]
fn idle_visual_system_completes() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // First, enter Launching to exercise that path.
    let angle = 0.0_f32;
    let power = 0.5;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run");
    app.update();

    // Now go back to Idle.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Idle;
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in Idle state");
    app.update();
}

/// Successive Launching calls at different power levels complete without panicking.
#[test]
fn launching_at_varying_power_levels() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = 0.0_f32;

    for power in [0.2, 0.5, 0.8, 1.0] {
        *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
        app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run");
        app.update();
    }
}
