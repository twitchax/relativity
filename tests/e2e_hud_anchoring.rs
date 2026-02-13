// E2E headless test: verifies that the HUD bar is anchored to the bottom 12%
// of the screen by inspecting the UiLayout boundary on the HudBar entity.
//
// This validates acceptance criterion uat-002 (PRD-0013):
// "HUD bar remains anchored to the bottom 12% of the screen".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use bevy_lunex::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::HudBar;

/// The HudBar boundary must span from y=88% to y=100% (bottom 12% of screen).
#[test]
fn hud_bar_anchored_to_bottom_twelve_percent() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let layout = app
        .world_mut()
        .query_filtered::<&UiLayout, With<HudBar>>()
        .single(app.world())
        .expect("HudBar should have a UiLayout")
        .clone();

    let expected = UiLayout::boundary().pos1(Rl((0.0, 88.0))).pos2(Rl(100.0)).pack();
    assert_eq!(layout, expected, "HudBar layout must anchor to the bottom 12% of the screen (y: 88%..100%)");
}
