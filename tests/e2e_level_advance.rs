// E2E headless test: verifies that completing Level 1 advances CurrentLevel
// via next() and re-enters AppState::InGame with the new level.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use relativity::{
    game::{fade::FadeState, levels::CurrentLevel, shared::types::PendingNextLevel},
    menu::MenuPlugin,
    shared::state::{AppState, GameState},
};

/// Build a gameplay app that also includes MenuPlugin so the
/// auto_advance_to_next_level system is registered.
fn build_app_with_menu() -> App {
    let mut app = common::gameplay::build_gameplay_app();
    app.add_plugins(MenuPlugin);
    app
}

/// After completing Level 1 (GameState::Finished), advancing CurrentLevel
/// via next() and inserting PendingNextLevel should cause the app to
/// transition through Menu back to InGame with CurrentLevel::TimeWarp.
#[test]
fn completing_level1_advances_to_next_level_and_reenters_ingame() {
    let mut app = build_app_with_menu();
    common::enter_game(&mut app);

    // Verify we start on Level 1.
    assert_eq!(*app.world().resource::<CurrentLevel>(), CurrentLevel::One);

    // Transition to Finished (simulating destination collision).
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Finished);
    app.update();
    assert_eq!(common::current_game_state(&app), GameState::Finished);

    // Exercise CurrentLevel::next() to advance the level and insert PendingNextLevel,
    // exactly as success_button_interaction does on a "Next Level" button press.
    let next = app.world().resource::<CurrentLevel>().next().expect("Level 1 should have a next level");
    *app.world_mut().resource_mut::<CurrentLevel>() = next;
    app.world_mut().insert_resource(PendingNextLevel);

    // Kick off the fade-out toward Menu.
    app.world_mut().resource_mut::<FadeState>().start_fade_out(AppState::Menu, GameState::Paused);

    // Run enough frames for:
    //   - fade-out to complete (~0.3s at 60fps = ~18 frames)
    //   - state transition to Menu (triggers auto_advance_to_next_level)
    //   - auto_advance starts another fade-out to InGame (~18 more frames)
    //   - state transition to InGame
    //   - fade-in (~18 frames)
    // Use generous budget to ensure all transitions complete.
    for _ in 0..120 {
        app.update();
    }

    // Assert CurrentLevel advanced to TimeWarp.
    assert_eq!(
        *app.world().resource::<CurrentLevel>(),
        CurrentLevel::TimeWarp,
        "CurrentLevel should have advanced to TimeWarp after completing Level 1"
    );

    // Assert AppState is back to InGame.
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::InGame,
        "AppState should be InGame after auto-advancing from Menu"
    );

    // Assert PendingNextLevel was consumed.
    assert!(app.world().get_resource::<PendingNextLevel>().is_none(), "PendingNextLevel should be consumed after auto-advancing");
}
