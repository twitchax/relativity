// E2E headless test: verifies that the HUD root dimension and position are
// synced to the camera's `ScalingMode::Fixed` projection rather than the
// pixel-based `UiFetchFromCamera`.
//
// bevy_lunex's built-in `UiFetchFromCamera` uses `camera.logical_viewport_size()`
// which returns pixel dimensions — these don't match the fixed world-space area
// when `ScalingMode::Fixed` is used. The custom `sync_hud_dimension_from_camera`
// system reads the projection directly and sets the HUD Dimension + Transform.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy_lunex::prelude::*;
use common::{build_gameplay_app, enter_game};
use relativity::game::hud::HudRoot;

/// The HUD root must NOT have `UiFetchFromCamera`, which uses pixel-based
/// viewport dimensions that don't account for `ScalingMode::Fixed`.
#[test]
fn hud_root_does_not_use_pixel_based_camera_fetch() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let has_fetch = app.world_mut().query_filtered::<Entity, (With<HudRoot>, With<UiFetchFromCamera<0>>)>().iter(app.world()).count();

    assert_eq!(
        has_fetch, 0,
        "HudRoot must NOT have UiFetchFromCamera — it uses pixel viewport size which doesn't account for ScalingMode::Fixed"
    );
}

/// When a camera with `ScalingMode::Fixed` exists, the sync system should
/// set the HUD root `Dimension` to match the fixed world-space area and
/// center the HUD root on the camera.
#[test]
fn hud_dimension_matches_fixed_camera_projection() {
    let mut app = build_gameplay_app();

    // Manually spawn a camera entity with the ScalingMode::Fixed projection.
    // In the headless test app, there's no DefaultPlugins camera, so we insert
    // just the components the sync system queries: Camera2d, Projection, Transform.
    let projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed { width: 1920.0, height: 1080.0 },
        ..OrthographicProjection::default_2d()
    });
    let cam_transform = Transform::from_xyz(960.0, 540.0, 0.0);
    app.world_mut().spawn((Camera2d, projection, cam_transform));

    enter_game(&mut app);

    // Run a few frames so the sync system has a chance to execute.
    for _ in 0..3 {
        app.update();
    }

    // Check the HUD root's Dimension matches the fixed camera area.
    let dimension = app
        .world_mut()
        .query_filtered::<&Dimension, With<HudRoot>>()
        .single(app.world())
        .expect("HudRoot should have a Dimension component");

    assert_eq!(
        **dimension,
        bevy::math::Vec2::new(1920.0, 1080.0),
        "HUD root Dimension must match ScalingMode::Fixed world-space area (1920×1080)"
    );

    // Check the HUD root's Transform is centered on the camera.
    let hud_transform = app
        .world_mut()
        .query_filtered::<&Transform, With<HudRoot>>()
        .single(app.world())
        .expect("HudRoot should have a Transform component");

    assert_eq!(hud_transform.translation, cam_transform.translation, "HUD root Transform must be centered on the camera position");
}

/// The HUD root must still have `UiLayoutRoot` so bevy_lunex processes
/// the layout tree. The sync system replaces only the dimension source,
/// not the layout root marker.
#[test]
fn hud_root_retains_layout_root() {
    let mut app = build_gameplay_app();
    enter_game(&mut app);

    let root_count = app.world_mut().query_filtered::<Entity, (With<HudRoot>, With<UiLayoutRoot>)>().iter(app.world()).count();

    assert_eq!(root_count, 1, "HudRoot must still have UiLayoutRoot for the layout tree to function");
}
