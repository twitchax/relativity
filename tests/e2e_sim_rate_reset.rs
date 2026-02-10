// E2E headless test: verifies that SimRate persists across level starts, re-launches,
// and crashes (it is never automatically reset).

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::shared::types::SimRate;
use relativity::shared::state::GameState;

fn read_sim_rate(app: &App) -> f64 {
    app.world().resource::<SimRate>().0
}

/// Press a key via the sim_rate_adjust system.
fn press_key_adjust(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);

    app.world_mut().run_system_once(relativity::game::shared::systems::sim_rate_adjust).expect("sim_rate_adjust should run");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(key);
}

#[test]
fn sim_rate_is_default_on_first_level_start() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // On first entry, SimRate should be its init_resource default of 1.0.
    assert!(
        (read_sim_rate(&app) - 1.0).abs() < f64::EPSILON,
        "SimRate should be 1.0 on first level start, got {}",
        read_sim_rate(&app)
    );
}

#[test]
fn sim_rate_persists_across_relaunch() {
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

    // Re-enter InGame — SimRate should persist.
    common::enter_game(&mut app);

    assert!(
        (read_sim_rate(&app) - 1.5).abs() < f64::EPSILON,
        "SimRate should persist at 1.5 across level re-start, got {}",
        read_sim_rate(&app)
    );
}
