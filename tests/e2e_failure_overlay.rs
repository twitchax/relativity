// E2E headless test: verifies that transitioning to GameState::Failed
// spawns a FailureOverlay entity, and that the failure timer auto-resets
// the state to GameState::Paused after ~1.5 seconds (90 frames at 60fps).

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use relativity::{
    game::shared::types::{FailureOverlay, FailureTimer},
    shared::state::GameState,
};

#[test]
fn failure_overlay_spawns_on_failed() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Failed.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Failed);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Failed);

    // Assert FailureOverlay entity exists.
    let overlay_count = app.world_mut().query_filtered::<Entity, With<FailureOverlay>>().iter(app.world()).count();
    assert_eq!(overlay_count, 1, "Expected exactly one FailureOverlay entity");

    // Assert FailureTimer resource was inserted.
    assert!(app.world().get_resource::<FailureTimer>().is_some(), "Expected FailureTimer resource to be inserted");
}

#[test]
fn failure_overlay_auto_resets_to_paused_after_delay() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Failed.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Failed);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Failed);

    // FailureOverlay should exist.
    let overlay_count = app.world_mut().query_filtered::<Entity, With<FailureOverlay>>().iter(app.world()).count();
    assert_eq!(overlay_count, 1);

    // Run frames to let the 1.5s timer expire (at 60fps = 90 frames).
    // Use a few extra frames to account for timing granularity.
    for _ in 0..95 {
        app.update();
    }

    // After the timer expires, GameState should transition to Paused.
    assert_eq!(common::current_game_state(&app), GameState::Paused, "Expected GameState to auto-reset to Paused after failure timer");

    // FailureOverlay should be despawned (OnExit(Failed) triggers despawn).
    let overlay_count = app.world_mut().query_filtered::<Entity, With<FailureOverlay>>().iter(app.world()).count();
    assert_eq!(overlay_count, 0, "Expected FailureOverlay to be despawned after reset");
}
