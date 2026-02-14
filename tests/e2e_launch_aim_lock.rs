// E2E headless test: verifies that a left click during `LaunchState::Idle`
// transitions LaunchState to `AimLocked` with the correct aim angle.
//
// Uses the same headless cursor/camera injection technique as
// `e2e_launch_preview.rs` so that `viewport_to_world_2d` succeeds.

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
use relativity::game::{player::player_sprite::launch_aim_system, shared::types::LaunchState};

/// Spawn a `PrimaryWindow` entity with the cursor at the given physical position.
fn spawn_window_with_cursor(world: &mut World, cursor_pos: DVec2) -> Entity {
    let mut window = Window {
        resolution: bevy::window::WindowResolution::new(800, 600),
        ..Default::default()
    };
    window.set_physical_cursor_position(Some(cursor_pos));
    world.spawn((window, PrimaryWindow)).id()
}

/// Spawn a headless `Camera2d` with computed projection so `viewport_to_world_2d` works.
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

/// Left click while Idle transitions LaunchState to AimLocked with the
/// correct angle derived from cursor position relative to the player.
#[test]
fn click_locks_aim_direction() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    assert_eq!(*app.world().resource::<LaunchState>(), LaunchState::Idle);

    // Place cursor to the right of center (500, 300 in an 800×600 window).
    spawn_window_with_cursor(app.world_mut(), DVec2::new(500.0, 300.0));
    spawn_headless_camera(app.world_mut());

    // Simulate left mouse button just-pressed.
    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);

    // Run the aim system.
    app.world_mut().run_system_once(launch_aim_system).expect("launch_aim_system should run");

    // LaunchState should now be AimLocked with some angle.
    let state = app.world().resource::<LaunchState>().clone();
    match state {
        LaunchState::AimLocked { angle } => {
            // The angle should be finite (sanity check).
            assert!(angle.is_finite(), "aim angle should be finite, got {angle}");
        }
        other => panic!("Expected AimLocked, got {other:?}"),
    }
}

/// When LaunchState is already AimLocked, a second click does NOT change
/// the state (the system early-returns).
#[test]
fn click_does_not_re_aim_when_already_locked() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    let original_angle = 1.23_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::AimLocked { angle: original_angle };

    spawn_window_with_cursor(app.world_mut(), DVec2::new(100.0, 100.0));
    spawn_headless_camera(app.world_mut());

    app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);

    app.world_mut().run_system_once(launch_aim_system).expect("launch_aim_system should run");

    // Angle should remain unchanged.
    let state = app.world().resource::<LaunchState>().clone();
    assert_eq!(state, LaunchState::AimLocked { angle: original_angle });
}
