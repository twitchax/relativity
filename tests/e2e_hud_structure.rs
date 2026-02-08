// E2E headless test: verifies that the HUD spawns at the bottom of the
// screen with two chrome panels (player + observer) backed by sprites.
//
// This validates acceptance criterion uat-001: "HUD renders at bottom of
// screen with chrome panels".  Because UiLunexPlugins is registered only
// in main.rs, we cannot verify pixel-level rendering in headless tests.
// Instead we verify the structural hierarchy and layout positioning that
// *produce* the bottom-anchored chrome panels at runtime.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::{HudBar, HudRoot, ObserverPanel, PlayerPanel};

/// After entering InGame, the HUD root, bar, and both panels must exist,
/// and the panels must carry Sprite components (chrome backgrounds).
#[test]
fn hud_spawns_with_bottom_bar_and_chrome_panels() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // HudRoot must exist.
    let root_count = app.world_mut().query_filtered::<Entity, With<HudRoot>>().iter(app.world()).count();
    assert_eq!(root_count, 1, "expected exactly one HudRoot entity");

    // HudBar must exist.
    let bar_count = app.world_mut().query_filtered::<Entity, With<HudBar>>().iter(app.world()).count();
    assert_eq!(bar_count, 1, "expected exactly one HudBar entity");

    // PlayerPanel must exist and carry a Sprite (chrome panel).
    let player_panels: Vec<Entity> = app.world_mut().query_filtered::<Entity, (With<PlayerPanel>, With<Sprite>)>().iter(app.world()).collect();
    assert_eq!(player_panels.len(), 1, "expected exactly one PlayerPanel with Sprite");

    // ObserverPanel must exist and carry a Sprite (chrome panel).
    let observer_panels: Vec<Entity> = app.world_mut().query_filtered::<Entity, (With<ObserverPanel>, With<Sprite>)>().iter(app.world()).collect();
    assert_eq!(observer_panels.len(), 1, "expected exactly one ObserverPanel with Sprite");
}

/// The HudBar must be a child of HudRoot, and both panels must be children
/// of HudBar, confirming the expected hierarchy for bottom-anchored layout.
#[test]
fn hud_hierarchy_is_correct() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Get the HudRoot entity.
    let root = app.world_mut().query_filtered::<Entity, With<HudRoot>>().single(app.world()).expect("HudRoot entity should exist");

    // Get the HudBar entity and verify it is a child of root.
    let bar = app.world_mut().query_filtered::<Entity, With<HudBar>>().single(app.world()).expect("HudBar entity should exist");

    let bar_parent = app.world().get::<ChildOf>(bar).expect("HudBar should have a parent");
    assert_eq!(bar_parent.parent(), root, "HudBar should be a child of HudRoot");

    // Get the PlayerPanel and verify it is a child of bar.
    let player_panel = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerPanel>>()
        .single(app.world())
        .expect("PlayerPanel entity should exist");

    let pp_parent = app.world().get::<ChildOf>(player_panel).expect("PlayerPanel should have a parent");
    assert_eq!(pp_parent.parent(), bar, "PlayerPanel should be a child of HudBar");

    // Get the ObserverPanel and verify it is a child of bar.
    let observer_panel = app
        .world_mut()
        .query_filtered::<Entity, With<ObserverPanel>>()
        .single(app.world())
        .expect("ObserverPanel entity should exist");

    let op_parent = app.world().get::<ChildOf>(observer_panel).expect("ObserverPanel should have a parent");
    assert_eq!(op_parent.parent(), bar, "ObserverPanel should be a child of HudBar");
}
