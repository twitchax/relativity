// E2E headless test: verifies that pressing -/Minus decreases SimRate by 0.25x, clamped to 0.25x.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::shared::types::SimRate;
use relativity::shared::state::GameState;

/// Press a key via the sim_rate_adjust system.
fn press_key_adjust(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);

    app.world_mut().run_system_once(relativity::game::shared::systems::sim_rate_adjust).expect("sim_rate_adjust should run");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(key);
}

fn read_sim_rate(app: &App) -> f64 {
    app.world().resource::<SimRate>().0
}

#[test]
fn minus_key_decreases_sim_rate_by_quarter() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON);

    // Press Minus → 0.75.
    press_key_adjust(&mut app, KeyCode::Minus);
    assert!((read_sim_rate(&app) - 0.75).abs() < f64::EPSILON);

    // Press Minus again → 0.50.
    press_key_adjust(&mut app, KeyCode::Minus);
    assert!((read_sim_rate(&app) - 0.50).abs() < f64::EPSILON);
}

#[test]
fn minus_key_clamps_at_min() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    // Press Minus 4 times: 1.0 → 0.75 → 0.50 → 0.25 → 0.25 (clamped).
    for _ in 0..4 {
        press_key_adjust(&mut app, KeyCode::Minus);
    }

    assert!((read_sim_rate(&app) - SimRate::MIN).abs() < f64::EPSILON);

    // One more press — should remain clamped at 0.25.
    press_key_adjust(&mut app, KeyCode::Minus);
    assert!((read_sim_rate(&app) - SimRate::MIN).abs() < f64::EPSILON);
}

#[test]
fn numpad_subtract_also_decreases_sim_rate() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    // NumpadSubtract should work just like Minus.
    press_key_adjust(&mut app, KeyCode::NumpadSubtract);
    assert!((read_sim_rate(&app) - 0.75).abs() < f64::EPSILON);
}
