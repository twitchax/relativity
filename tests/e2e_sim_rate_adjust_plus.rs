// E2E headless test: verifies that pressing +/Equal increases SimRate by 0.25x, clamped to 2.00x.

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
fn plus_key_increases_sim_rate_by_quarter() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Get to Running state.
    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON);

    // Press Equal (plus) → 1.25.
    press_key_adjust(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - 1.25).abs() < f64::EPSILON);

    // Press Equal again → 1.50.
    press_key_adjust(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - 1.50).abs() < f64::EPSILON);
}

#[test]
fn plus_key_clamps_at_max() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    // Press Equal 5 times: 1.0 → 1.25 → 1.50 → 1.75 → 2.00 → 2.00 (clamped).
    for _ in 0..5 {
        press_key_adjust(&mut app, KeyCode::Equal);
    }

    assert!((read_sim_rate(&app) - SimRate::MAX).abs() < f64::EPSILON);

    // One more press — should remain clamped at 2.00.
    press_key_adjust(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - SimRate::MAX).abs() < f64::EPSILON);
}

#[test]
fn numpad_add_also_increases_sim_rate() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    // NumpadAdd should work just like Equal.
    press_key_adjust(&mut app, KeyCode::NumpadAdd);
    assert!((read_sim_rate(&app) - 1.25).abs() < f64::EPSILON);
}
