// E2E test: verifies that during the Launching phase the radial arc around the
// player fills proportionally to power and uses the correct color gradient
// (cyan at low power → orange → red at high power).
//
// Gizmos are immediate-mode GPU draws and cannot be queried as ECS entities.
// We verify by:
//  1. Running `launch_visual_system` at several power levels (proving the arc
//     code path executes without panicking).
//  2. Asserting the pure helpers (`map_power_nonlinear`, `power_to_color`) that
//     drive arc fill and color produce the expected gradient at those levels.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::ecs::system::RunSystemOnce;
use relativity::game::player::player_sprite::{launch_visual_system, map_power_nonlinear, power_to_color};
use relativity::game::shared::types::LaunchState;

/// Power levels used for the gradient sweep.
const POWERS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// Running `launch_visual_system` in the Launching state at progressively
/// higher power levels exercises the filled‐arc + color gradient code path.
#[test]
fn arc_fills_at_increasing_power_levels_without_panic() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let angle = std::f32::consts::FRAC_PI_4;

    for &power in &POWERS {
        *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };
        app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run without panic");
        app.update();
    }
}

/// The mapped power fraction increases monotonically, meaning the filled arc
/// angle grows with higher raw power.
#[test]
fn arc_fill_fraction_increases_with_power() {
    let mapped: Vec<f32> = POWERS.iter().map(|&p| map_power_nonlinear(p)).collect();

    for window in mapped.windows(2) {
        assert!(
            window[1] > window[0],
            "mapped power should increase: f({}) = {} should be < f(next) = {}",
            window[0],
            window[0],
            window[1],
        );
    }
}

/// The color gradient transitions from cyan (low power) through warm tones to
/// red (full power), verifying the visual feedback is correct.
#[test]
fn color_gradient_transitions_cyan_to_red() {
    let colors: Vec<_> = POWERS.iter().map(|&p| power_to_color(map_power_nonlinear(p)).to_srgba()).collect();

    // Lowest mapped power: predominantly cyan (high green+blue, low red).
    let low = &colors[0];
    assert!(low.blue > 0.5, "low power should have significant blue (cyan)");
    assert!(low.red < 0.5, "low power should have low red");

    // Highest mapped power (1.0): red dominant, no blue.
    let high = colors.last().unwrap();
    assert!(high.red > 0.9, "full power should be nearly pure red");
    assert!(high.blue < 0.01, "full power should have no blue");

    // Red channel should generally increase across the sweep.
    for window in colors.windows(2) {
        assert!(window[1].red >= window[0].red - 0.01, "red channel should not decrease significantly across power sweep",);
    }
}
