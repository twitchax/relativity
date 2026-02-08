// E2E headless test: verifies that transitioning to GameState::Finished
// spawns a SuccessOverlay entity containing a NextLevelButton with "Next Level" text.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use relativity::{
    game::shared::types::{NextLevelButton, SuccessOverlay},
    shared::state::GameState,
};

#[test]
fn success_overlay_spawns_on_finished_with_next_level_button() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Finished.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Finished);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Finished);

    // Assert SuccessOverlay entity exists.
    let overlay_count = app.world_mut().query_filtered::<Entity, With<SuccessOverlay>>().iter(app.world()).count();
    assert_eq!(overlay_count, 1, "Expected exactly one SuccessOverlay entity");

    // Assert NextLevelButton entity exists.
    let button_count = app.world_mut().query_filtered::<Entity, (With<NextLevelButton>, With<Button>)>().iter(app.world()).count();
    assert_eq!(button_count, 1, "Expected exactly one NextLevelButton");

    // Assert "Next Level" text exists (Level One has a next level, so text should be "Next Level").
    let has_next_level_text = app.world_mut().query::<&Text>().iter(app.world()).any(|text| text.0 == "Next Level");
    assert!(has_next_level_text, "Expected a Text node with content \"Next Level\"");
}
