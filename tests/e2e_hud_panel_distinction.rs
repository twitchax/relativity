// E2E headless test: verifies that the player panel (left) and observer panel
// (right) use distinct sprite assets, ensuring they are visually distinguishable.
//
// This validates acceptance criterion uat-003: "Player panel (left) and observer
// panel (right) are visually distinct with new panel art".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::{ObserverPanel, PlayerPanel};

/// Both panels must exist with Sprite components, and their image handles must
/// differ — proving they use separate panel art assets.
#[test]
fn player_and_observer_panels_use_distinct_sprites() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let player_image = app
        .world_mut()
        .query_filtered::<&Sprite, With<PlayerPanel>>()
        .single(app.world())
        .expect("PlayerPanel with Sprite should exist")
        .image
        .clone();

    let observer_image = app
        .world_mut()
        .query_filtered::<&Sprite, With<ObserverPanel>>()
        .single(app.world())
        .expect("ObserverPanel with Sprite should exist")
        .image
        .clone();

    assert_ne!(player_image, observer_image, "player and observer panels must use different sprite assets");
}
