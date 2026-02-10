// E2E headless test: verifies that SimRate resets to 1.00x on level start and re-launch.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::shared::types::SimRate;
use relativity::shared::state::GameState;

fn read_sim_rate(app: &App) -> f64 {
    app.world().resource::<SimRate>().0
}

/// Mutate SimRate to a non-default value directly.
fn set_sim_rate(app: &mut App, value: f64) {
    app.world_mut().resource_mut::<SimRate>().0 = value;
}

/// Press a key via the sim_rate_adjust system.
fn press_key_adjust(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);

    app.world_mut().run_system_once(relativity::game::shared::systems::sim_rate_adjust).expect("sim_rate_adjust should run");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(key);
}

#[test]
fn sim_rate_is_default_on_level_start() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // On entering InGame, reset_sim_rate runs → SimRate should be 1.0.
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON, "SimRate should be 1.0 on level start, got {}", read_sim_rate(&app));
}

#[test]
fn sim_rate_resets_on_relaunch() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running);

    // Increase rate to 1.50x while running.
    press_key_adjust(&mut app, KeyCode::Equal);
    press_key_adjust(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - 1.5).abs() < f64::EPSILON, "SimRate should be 1.5 after two presses, got {}", read_sim_rate(&app));

    // Exit to menu and re-enter the game (simulates level re-start).
    app.world_mut()
        .resource_mut::<NextState<relativity::shared::state::AppState>>()
        .set(relativity::shared::state::AppState::Menu);
    app.update();

    // Re-enter InGame — reset_sim_rate fires again.
    common::enter_game(&mut app);

    assert!(
        (read_sim_rate(&app) - 1.0).abs() < f64::EPSILON,
        "SimRate should reset to 1.0 on level re-start, got {}",
        read_sim_rate(&app)
    );
}

#[test]
fn reset_sim_rate_system_restores_default() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Manually set SimRate to a non-default value.
    set_sim_rate(&mut app, 1.75);
    assert!((read_sim_rate(&app) - 1.75).abs() < f64::EPSILON);

    // Directly invoke reset_sim_rate — the same system registered on OnEnter(InGame).
    app.world_mut().run_system_once(relativity::game::shared::systems::reset_sim_rate).expect("reset_sim_rate should run");

    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON, "SimRate should be 1.0 after reset, got {}", read_sim_rate(&app));
}
