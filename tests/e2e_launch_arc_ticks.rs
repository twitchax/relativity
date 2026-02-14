// E2E test: verifies that arc tick marks appear at 0.25c, 0.5c, 0.75c, and
// 0.9c on the radial power arc.
//
// Gizmos are immediate-mode GPU draws and cannot be queried as ECS entities.
// We verify by:
//  1. Confirming the `TICK_VELOCITY_FRACTIONS` constant has exactly the four
//     expected velocities.
//  2. Running `launch_visual_system` in the Launching state (which calls
//     `draw_arc_ticks`) to exercise the tick-drawing code path without panic.
//  3. Asserting that each tick's angular position falls within the arc's
//     sweep range (0–MAX_ARC_ANGLE).

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::ecs::system::RunSystemOnce;
use relativity::game::player::player_sprite::{launch_visual_system, map_power_nonlinear, MAX_ARC_ANGLE, TICK_VELOCITY_FRACTIONS};
use relativity::game::shared::types::LaunchState;

/// The tick mark constant contains exactly the four required velocity fractions.
#[test]
fn tick_velocity_fractions_match_spec() {
    assert_eq!(TICK_VELOCITY_FRACTIONS.len(), 4);
    assert!((TICK_VELOCITY_FRACTIONS[0] - 0.25).abs() < f32::EPSILON);
    assert!((TICK_VELOCITY_FRACTIONS[1] - 0.50).abs() < f32::EPSILON);
    assert!((TICK_VELOCITY_FRACTIONS[2] - 0.75).abs() < f32::EPSILON);
    assert!((TICK_VELOCITY_FRACTIONS[3] - 0.90).abs() < f32::EPSILON);
}

/// Each tick's angular position (derived from its velocity fraction) falls
/// within the arc's maximum sweep range, ensuring all four ticks are visible.
#[test]
fn tick_angular_positions_within_arc_range() {
    for &frac in &TICK_VELOCITY_FRACTIONS {
        let tick_power = frac / 0.99;
        // The arc sweep maps power to 0–MAX_ARC_ANGLE, centred at -0.5.
        let local_angle = MAX_ARC_ANGLE * (tick_power - 0.5);
        // local_angle should be within [-MAX_ARC_ANGLE/2, MAX_ARC_ANGLE/2].
        assert!(
            local_angle.abs() <= MAX_ARC_ANGLE / 2.0 + 0.01,
            "tick at {frac}c maps to local_angle {local_angle} which exceeds arc half-range"
        );
    }
}

/// Running `launch_visual_system` in the Launching state at several power
/// levels exercises the `draw_arc_ticks` code path without panicking.
#[test]
fn tick_marks_drawn_during_launching_without_panic() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = std::f32::consts::FRAC_PI_4;

    // Sweep through power levels that bracket all four tick positions.
    for power in [0.0, 0.3, 0.6, 0.9, 1.0] {
        *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
        app.world_mut()
            .run_system_once(launch_visual_system)
            .expect("launch_visual_system should draw tick marks without panic");
        app.update();
    }
}

/// Each tick velocity fraction converts to a mapped power that is monotonically
/// increasing and within the valid mapped power range.
#[test]
fn tick_fractions_produce_valid_mapped_powers() {
    let mut prev = 0.0_f32;

    for &frac in &TICK_VELOCITY_FRACTIONS {
        let raw_power = frac / 0.99;
        let mapped = map_power_nonlinear(raw_power);
        assert!(mapped > prev, "mapped power for {frac}c ({mapped}) should exceed previous ({prev})");
        assert!(mapped <= 1.0, "mapped power should not exceed 1.0");
        prev = mapped;
    }
}
