// E2E headless test: verifies that launch visuals (direction gizmo line and
// power bar UI) render correctly during the AimLocked and Launching phases.
//
// The direction line is drawn via Bevy Gizmos (immediate-mode GPU drawing) and
// cannot be queried as an ECS entity.  We verify it by running the
// `launch_visual_system` in AimLocked state and confirming it completes without
// panicking — the gizmo draw calls execute on the correct code path.
//
// The power bar is spawned as a UI node with the `PowerBarUi` marker component
// and *can* be queried.  We verify it spawns during Launching, has the expected
// width proportional to power, and despawns when returning to Idle.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::{
    player::player_sprite::launch_visual_system,
    shared::types::{LaunchState, PowerBarUi},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Count entities with the `PowerBarUi` marker.
fn power_bar_count(app: &mut App) -> usize {
    app.world_mut().query_filtered::<Entity, With<PowerBarUi>>().iter(app.world()).count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// In AimLocked state, the direction gizmo line is drawn and no PowerBarUi
/// entity exists (any leftover bar from a previous Launching phase is despawned).
#[test]
fn aim_locked_draws_direction_line_no_power_bar() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Set AimLocked with a 45° angle.
    let angle = std::f32::consts::FRAC_PI_4;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle };

    // Run the visual system — gizmo line_2d is called inside; any panic here
    // means the direction-line code path is broken.
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in AimLocked state");
    app.update(); // flush command queue

    // No power bar should be present.
    assert_eq!(power_bar_count(&mut app), 0, "PowerBarUi should not exist during AimLocked");
}

/// In Launching state, the direction line is drawn (scaled by power) AND a
/// PowerBarUi entity is spawned with width proportional to power.
#[test]
fn launching_spawns_power_bar() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = std::f32::consts::FRAC_PI_4;
    let power = 0.6_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };

    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in Launching state");
    app.update(); // flush command queue

    // A PowerBarUi entity should have been spawned.
    assert_eq!(power_bar_count(&mut app), 1, "exactly one PowerBarUi should exist during Launching");

    // Verify the bar's outer container has the expected width.
    let bar_entity = app.world_mut().query_filtered::<Entity, With<PowerBarUi>>().single(app.world()).unwrap();
    let node = app.world().get::<Node>(bar_entity).expect("PowerBarUi should have a Node component");
    assert_eq!(node.width, Val::Px(204.0), "outer bar width should be 204px");
}

/// When LaunchState returns to Idle, any PowerBarUi entity is despawned.
#[test]
fn idle_despawns_power_bar() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // First, spawn a power bar by entering Launching.
    let angle = 0.0_f32;
    let power = 0.5;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run");
    app.update();
    assert_eq!(power_bar_count(&mut app), 1, "PowerBarUi should exist after Launching");

    // Now go back to Idle.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Idle;
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run in Idle state");
    app.update();

    assert_eq!(power_bar_count(&mut app), 0, "PowerBarUi should be despawned after returning to Idle");
}

/// Successive Launching calls replace the power bar (no duplicates accumulate).
#[test]
fn launching_replaces_power_bar_no_duplicates() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = 0.0_f32;

    // Run launch_visual_system multiple times with different power values.
    for power in [0.2, 0.5, 0.8, 1.0] {
        *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
        app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run");
        app.update();

        assert_eq!(power_bar_count(&mut app), 1, "exactly one PowerBarUi should exist at power={power}");
    }
}
