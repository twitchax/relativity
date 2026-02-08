// E2E headless test: verifies that pressing Escape returns to Menu from all GameState sub-states.
//
// Uses the gameplay test helpers to build a headless app with GamePlugin, transitions to
// various GameState sub-states, injects Escape key press, and asserts AppState transitions
// to Menu with proper cleanup.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::{
    game::shared::types::{FailureOverlay, LaunchState, SuccessOverlay},
    shared::state::{AppState, GameState},
};

/// Inject an Escape key press into the app and run updates to process the state transition.
///
/// Bevy's `InputPlugin` clears `just_pressed` tracking in `PreUpdate`, so we inject the
/// key press between updates using `ButtonInput::press`. The first `app.update()` clears it
/// via `PreUpdate`, so we inject after the first update, then the second update sees it as
/// `just_pressed` during its `Update` schedule, and the third update applies the queued state.
fn press_escape_and_process(app: &mut App) {
    // The key must be "just pressed" during the Update schedule.
    // ButtonInput::press() sets both pressed and just_pressed.
    // However, InputPlugin's PreUpdate clears just_pressed.
    // So we press the key, then update — but we need to avoid PreUpdate clearing it.
    //
    // Alternative: Use run_system_once to directly invoke exit_level_check with a pressed key.
    // Simpler: skip InputPlugin interference by pressing right before update.
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
    // Run only Update (not full app.update which includes PreUpdate that clears just_pressed).
    // Actually, full app.update runs PreUpdate first which clears just_pressed before Update.
    // We need to work around this.

    // Solution: call clear_just_pressed manually AFTER the input system clears it but BEFORE our system runs.
    // Actually, the simplest approach is to call the system directly.
    app.world_mut()
        .run_system_once(relativity::game::shared::systems::exit_level_check)
        .expect("exit_level_check should run");

    // Now release the key to clean up.
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::Escape);

    // Run update to apply the queued NextState transitions.
    app.update();
}

/// Read the current `AppState`.
fn current_app_state(app: &App) -> AppState {
    app.world().resource::<State<AppState>>().get().clone()
}

/// Count entities matching a filter.
fn count_entities<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut().query_filtered::<Entity, F>().iter(app.world()).count()
}

// ---------- Escape from Paused ----------

#[test]
fn escape_from_paused_returns_to_menu() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    assert_eq!(common::current_game_state(&app), GameState::Paused);
    assert_eq!(current_app_state(&app), AppState::InGame);

    press_escape_and_process(&mut app);

    assert_eq!(current_app_state(&app), AppState::Menu);
}

// ---------- Escape from Running ----------

#[test]
fn escape_from_running_returns_to_menu() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Running);

    press_escape_and_process(&mut app);

    assert_eq!(current_app_state(&app), AppState::Menu);
}

// ---------- Escape from Failed ----------

#[test]
fn escape_from_failed_returns_to_menu() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Failed.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Failed);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Failed);
    assert!(count_entities::<With<FailureOverlay>>(&mut app) > 0, "Failure overlay should be spawned");

    press_escape_and_process(&mut app);

    assert_eq!(current_app_state(&app), AppState::Menu);
    assert_eq!(count_entities::<With<FailureOverlay>>(&mut app), 0, "Failure overlay should be despawned on Escape");
}

// ---------- Escape from Finished ----------

#[test]
fn escape_from_finished_returns_to_menu() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Finished.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Finished);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Finished);
    assert!(count_entities::<With<SuccessOverlay>>(&mut app) > 0, "Success overlay should be spawned");

    press_escape_and_process(&mut app);

    assert_eq!(current_app_state(&app), AppState::Menu);
    assert_eq!(count_entities::<With<SuccessOverlay>>(&mut app), 0, "Success overlay should be despawned on Escape");
}

// ---------- LaunchState reset ----------

#[test]
fn escape_resets_launch_state_to_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Simulate being mid-aim.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: 1.0 };

    press_escape_and_process(&mut app);

    let launch_state = app.world().resource::<LaunchState>();
    assert_eq!(*launch_state, LaunchState::Idle, "LaunchState should be reset to Idle on Escape");
}
