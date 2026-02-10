// E2E headless test: verifies that pressing Space toggles between Running and SimPaused.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::shared::state::GameState;

/// Press Space via the sim_pause_toggle system.
fn press_space_toggle(app: &mut App) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);

    app.world_mut()
        .run_system_once(relativity::game::shared::systems::sim_pause_toggle)
        .expect("sim_pause_toggle should run");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::Space);

    // Process queued state transition.
    app.update();
}

#[test]
fn space_pauses_running_and_resumes() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Get to Running state.
    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running);

    // Press Space → should pause.
    press_space_toggle(&mut app);
    assert_eq!(common::current_game_state(&app), GameState::SimPaused);

    // Press Space again → should resume.
    press_space_toggle(&mut app);
    assert_eq!(common::current_game_state(&app), GameState::Running);
}

#[test]
fn space_does_nothing_in_paused_state() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Default GameState after entering game is Paused (launch aiming).
    assert_eq!(common::current_game_state(&app), GameState::Paused);

    press_space_toggle(&mut app);

    // Should still be Paused — Space only toggles Running <-> SimPaused.
    assert_eq!(common::current_game_state(&app), GameState::Paused);
}
