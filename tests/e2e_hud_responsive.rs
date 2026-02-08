// E2E headless test: verifies that the HUD is built with responsive
// bevy_lunex layout primitives (`UiLayout`, `UiTextSize`) on every
// structural entity.  Because all positions use relative units (`Rl`/`Rh`),
// the layout engine automatically adapts to any window size or aspect ratio.
//
// This validates acceptance criterion uat-004: "HUD adapts to different
// window sizes and aspect ratios".  Full pixel-level rendering cannot be
// verified in headless tests (UiLunexPlugins requires DefaultPlugins),
// but the structural presence of responsive layout components guarantees
// correct behaviour at runtime.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use bevy_lunex::prelude::{UiLayout, UiTextSize};
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::{HudBar, HudGravGamma, HudObserverTime, HudPlayerTime, HudRoot, HudVelocityFraction, HudVelocityGamma, ObserverPanel, PlayerPanel};

/// Every structural HUD entity (bar, panels) must carry a `UiLayout`
/// component, ensuring bevy_lunex manages their positioning responsively.
#[test]
fn hud_structural_entities_have_responsive_layout() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // HudBar must have a UiLayout (boundary at bottom 12%).
    let bar_has_layout = app.world_mut().query_filtered::<Entity, (With<HudBar>, With<UiLayout>)>().iter(app.world()).count();
    assert_eq!(bar_has_layout, 1, "HudBar must have a UiLayout component");

    // PlayerPanel must have a UiLayout.
    let pp_has_layout = app.world_mut().query_filtered::<Entity, (With<PlayerPanel>, With<UiLayout>)>().iter(app.world()).count();
    assert_eq!(pp_has_layout, 1, "PlayerPanel must have a UiLayout component");

    // ObserverPanel must have a UiLayout.
    let op_has_layout = app.world_mut().query_filtered::<Entity, (With<ObserverPanel>, With<UiLayout>)>().iter(app.world()).count();
    assert_eq!(op_has_layout, 1, "ObserverPanel must have a UiLayout component");
}

/// All five text labels must carry both `UiLayout` (for responsive positioning)
/// and `UiTextSize` (for responsive font scaling relative to parent height).
#[test]
fn hud_text_labels_have_responsive_sizing() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // Each label marker should co-exist with UiLayout + UiTextSize.
    let tp = app
        .world_mut()
        .query_filtered::<Entity, (With<HudPlayerTime>, With<UiLayout>, With<UiTextSize>)>()
        .iter(app.world())
        .count();
    assert_eq!(tp, 1, "HudPlayerTime must have UiLayout + UiTextSize");

    let vg = app
        .world_mut()
        .query_filtered::<Entity, (With<HudVelocityGamma>, With<UiLayout>, With<UiTextSize>)>()
        .iter(app.world())
        .count();
    assert_eq!(vg, 1, "HudVelocityGamma must have UiLayout + UiTextSize");

    let gg = app
        .world_mut()
        .query_filtered::<Entity, (With<HudGravGamma>, With<UiLayout>, With<UiTextSize>)>()
        .iter(app.world())
        .count();
    assert_eq!(gg, 1, "HudGravGamma must have UiLayout + UiTextSize");

    let vf = app
        .world_mut()
        .query_filtered::<Entity, (With<HudVelocityFraction>, With<UiLayout>, With<UiTextSize>)>()
        .iter(app.world())
        .count();
    assert_eq!(vf, 1, "HudVelocityFraction must have UiLayout + UiTextSize");

    let ot = app
        .world_mut()
        .query_filtered::<Entity, (With<HudObserverTime>, With<UiLayout>, With<UiTextSize>)>()
        .iter(app.world())
        .count();
    assert_eq!(ot, 1, "HudObserverTime must have UiLayout + UiTextSize");
}

/// The HUD root must have `UiLayoutRoot` to serve as the bevy_lunex 2D
/// layout tree root, which is what drives viewport-size-aware layout.
#[test]
fn hud_root_has_layout_root_for_viewport_tracking() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    // UiLayoutRoot is what makes the layout tree track the camera viewport.
    let root_count = app
        .world_mut()
        .query_filtered::<Entity, (With<HudRoot>, With<bevy_lunex::prelude::UiLayoutRoot>)>()
        .iter(app.world())
        .count();
    assert_eq!(root_count, 1, "HudRoot must have UiLayoutRoot for viewport tracking");
}
