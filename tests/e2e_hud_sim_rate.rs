// E2E headless test: verifies that the HUD displays the current simulation
// rate in the observer (right) panel and updates when SimRate changes.
//
// This validates acceptance criterion uat-004: "HUD displays current
// simulation rate in right panel (always visible)".

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::HudSimRate;
use relativity::game::shared::types::SimRate;

/// After entering InGame, the observer panel must contain exactly one
/// `HudSimRate` entity.
#[test]
fn observer_panel_has_sim_rate_label() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let count = app.world_mut().query_filtered::<Entity, With<HudSimRate>>().iter(app.world()).count();
    assert_eq!(count, 1, "expected exactly one HudSimRate label");
}

/// At spawn, the sim rate label should show the default value `r = 1.00×`.
#[test]
fn sim_rate_label_shows_default_value() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let text = app.world_mut().query_filtered::<&Text2d, With<HudSimRate>>().single(app.world()).unwrap().0.clone();
    assert_eq!(text, "r = 1.00×", "initial sim rate text: {text}");
}

/// When SimRate is changed, the HUD label should reflect the new value
/// after an update tick.
#[test]
fn sim_rate_label_updates_when_rate_changes() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Change SimRate to 1.50.
    app.world_mut().resource_mut::<SimRate>().0 = 1.5;
    app.update();

    let text = app.world_mut().query_filtered::<&Text2d, With<HudSimRate>>().single(app.world()).unwrap().0.clone();
    assert_eq!(text, "r = 1.50×", "sim rate text after change: {text}");
}
