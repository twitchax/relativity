// E2E headless test: verifies that speed controls (+/-) only apply while GameState::Running.
// Speed adjustments must not take effect during Paused (launch-aim) or SimPaused states.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::message::Messages, input::keyboard::KeyboardInput, prelude::*};
use relativity::game::shared::types::SimRate;
use relativity::shared::state::GameState;

fn read_sim_rate(app: &App) -> f64 {
    app.world().resource::<SimRate>().0
}

/// Send a key-press event and run one full app update (respects schedule run conditions).
/// Uses `Messages<KeyboardInput>` so that the press is processed by `keyboard_input_system`
/// in `PreUpdate`, making `just_pressed` available to systems in `Update`.
fn press_key_and_update(app: &mut App, key: KeyCode) {
    // Send press event.
    app.world_mut().resource_mut::<Messages<KeyboardInput>>().write(KeyboardInput {
        key_code: key,
        logical_key: bevy::input::keyboard::Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: bevy::input::ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });

    app.update();

    // Send release event so the key doesn't stay pressed.
    app.world_mut().resource_mut::<Messages<KeyboardInput>>().write(KeyboardInput {
        key_code: key,
        logical_key: bevy::input::keyboard::Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
        state: bevy::input::ButtonState::Released,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });

    app.update();
}

#[test]
fn speed_controls_ignored_during_paused_state() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Default state after entering game is Paused (launch-aim).
    assert_eq!(common::current_game_state(&app), GameState::Paused);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON);

    // Press Equal (+) during Paused — SimRate must not change.
    press_key_and_update(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON, "SimRate should remain 1.0 during Paused state");

    // Press Minus (-) during Paused — SimRate must not change.
    press_key_and_update(&mut app, KeyCode::Minus);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON, "SimRate should remain 1.0 during Paused state");
}

#[test]
fn speed_controls_ignored_during_sim_paused_state() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Get to Running state, then transition to SimPaused.
    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();
    assert_eq!(common::current_game_state(&app), GameState::Running);

    // Transition to SimPaused.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::SimPaused);
    app.update();
    assert_eq!(common::current_game_state(&app), GameState::SimPaused);

    let rate_before = read_sim_rate(&app);

    // Press Equal (+) during SimPaused — SimRate must not change.
    press_key_and_update(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - rate_before).abs() < f64::EPSILON, "SimRate should not change during SimPaused state");

    // Press Minus (-) during SimPaused — SimRate must not change.
    press_key_and_update(&mut app, KeyCode::Minus);
    assert!((read_sim_rate(&app) - rate_before).abs() < f64::EPSILON, "SimRate should not change during SimPaused state");
}

#[test]
fn speed_controls_work_during_running_state() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();
    assert_eq!(common::current_game_state(&app), GameState::Running);
    assert!((read_sim_rate(&app) - 1.0).abs() < f64::EPSILON);

    // Press Equal (+) during Running — SimRate should increase.
    press_key_and_update(&mut app, KeyCode::Equal);
    assert!((read_sim_rate(&app) - 1.25).abs() < f64::EPSILON, "SimRate should increase to 1.25 during Running state");
}
