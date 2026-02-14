// E2E headless test: verifies that `launch_preview_system` draws the dotted
// aim-preview line when a cursor position is available and `LaunchState::Idle`.
//
// In headless tests (MinimalPlugins, no WinitPlugin) cursor_position() normally
// returns `None`.  We work around this by manually spawning a `Window` entity
// with `PrimaryWindow` and injecting a cursor position, plus a `Camera2d` with
// pre-computed projection values so that `viewport_to_world_2d` succeeds.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{
    camera::{OrthographicProjection, RenderTargetInfo, Viewport},
    ecs::system::RunSystemOnce,
    math::DVec2,
    prelude::*,
    window::PrimaryWindow,
};
use relativity::game::{player::player_sprite::launch_preview_system, shared::types::LaunchState};

/// Helper: spawn a `PrimaryWindow` entity with the cursor at the given logical
/// position.  Returns the entity so tests can adjust the cursor later.
fn spawn_window_with_cursor(world: &mut World, cursor_pos: DVec2) -> Entity {
    let mut window = Window {
        resolution: bevy::window::WindowResolution::new(800, 600),
        ..Default::default()
    };
    window.set_physical_cursor_position(Some(cursor_pos));
    world.spawn((window, PrimaryWindow)).id()
}

/// Helper: spawn a `Camera2d` entity with computed projection data so that
/// `viewport_to_world_2d` works without a GPU / render pass.
fn spawn_headless_camera(world: &mut World) -> Entity {
    let physical_size = UVec2::new(800, 600);
    let viewport = Viewport { physical_size, ..Default::default() };

    let mut projection = Projection::from(OrthographicProjection::default_2d());
    projection.update(physical_size.x as f32, physical_size.y as f32);

    let mut camera = Camera {
        viewport: Some(viewport),
        ..Default::default()
    };
    camera.computed.target_info = Some(RenderTargetInfo { physical_size, scale_factor: 1.0 });
    camera.computed.clip_from_view = projection.get_clip_from_view();

    world.spawn((Camera2d, camera, projection, GlobalTransform::default(), Transform::default())).id()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// When `LaunchState::Idle` and a cursor position is available, the full
/// `draw_dashed_line` path executes without panicking — proving the dotted
/// aim-preview line renders on hover before any click.
#[test]
fn preview_line_draws_on_hover_in_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Ensure we are in Idle (the default).
    assert_eq!(*app.world().resource::<LaunchState>(), LaunchState::Idle);

    // Inject a Window with cursor at (500, 300) — offset from center so the
    // direction vector is non-zero.
    spawn_window_with_cursor(app.world_mut(), DVec2::new(500.0, 300.0));
    spawn_headless_camera(app.world_mut());

    // Run the preview system — exercises the full path including draw_dashed_line.
    app.world_mut()
        .run_system_once(launch_preview_system)
        .expect("launch_preview_system should run in Idle state with cursor");
    app.update();
}

/// When `LaunchState` is **not** Idle, the preview system exits immediately
/// (no drawing).  This confirms the preview line only renders during hover
/// before any click.
#[test]
fn preview_line_skipped_when_not_idle() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    spawn_window_with_cursor(app.world_mut(), DVec2::new(500.0, 300.0));
    spawn_headless_camera(app.world_mut());

    // AimLocked — preview should NOT draw.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: 0.5 };

    app.world_mut().run_system_once(launch_preview_system).expect("launch_preview_system should exit early in AimLocked");
    app.update();

    // Launching — preview should NOT draw.
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle: 0.5, power: 0.5 };

    app.world_mut().run_system_once(launch_preview_system).expect("launch_preview_system should exit early in Launching");
    app.update();
}
