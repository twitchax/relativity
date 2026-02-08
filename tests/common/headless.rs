#![allow(dead_code)]

//! Headless render helpers for E2E tests.
//!
//! Provides a `build_headless_render_app()` that mirrors the real game's
//! `main()` setup — `DefaultPlugins` with `WinitPlugin` disabled and no
//! primary window — plus an offscreen camera that renders to an `Image` asset.
//!
//! The offscreen render target is stored as the `OffscreenRenderTarget` resource
//! so that screenshot-capture code can reference it.

use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    window::ExitCondition,
    winit::WinitPlugin,
};
use relativity::{
    game::{levels::CurrentLevel, GamePlugin},
    menu::MenuPlugin,
    shared::state::AppState,
};

// ------------------------------------------------------------------
// Resources
// ------------------------------------------------------------------

/// Stores the handle and dimensions of the offscreen image render target
/// so that other systems (e.g. screenshot capture) can reference it.
#[derive(Resource)]
pub struct OffscreenRenderTarget {
    pub handle: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

// ------------------------------------------------------------------
// App builder
// ------------------------------------------------------------------

/// Build a headless Bevy app in the same way `main()` does, but without
/// a real window or display server:
///
/// * `DefaultPlugins` with `WinitPlugin` disabled and no primary window.
/// * `MenuPlugin` + `GamePlugin` (full game logic).
/// * An offscreen 2-D camera rendering to an `Image` asset.
///
/// Call [`wait_for_plugins`] after this to finish GPU/plugin initialization
/// before calling `app.update()`.
pub fn build_headless_render_app() -> App {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<CurrentLevel>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        )
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .init_state::<AppState>()
        .add_systems(Startup, spawn_offscreen_camera);

    app
}

// ------------------------------------------------------------------
// Plugin lifecycle helpers
// ------------------------------------------------------------------

/// Drive the plugin lifecycle to completion.
///
/// The render plugin initializes the GPU asynchronously; `finish()` and
/// `cleanup()` ensure all plugin lifecycle hooks have run before we start
/// calling `app.update()`.
pub fn wait_for_plugins(app: &mut App) {
    while app.plugins_state() != bevy::app::PluginsState::Cleaned {
        if app.plugins_state() == bevy::app::PluginsState::Ready {
            app.finish();
            app.cleanup();
        }
    }
}

/// Suppress `RUST_BACKTRACE` to prevent wgpu / Metal hangs on macOS.
///
/// `cargo-make` sets `RUST_BACKTRACE=full` by default.  Any non-zero value
/// causes wgpu/Bevy's Metal render graph to hang on macOS because backtrace
/// symbolication contends with GPU driver internals.
pub fn suppress_backtrace() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "0");
    }
}

// ------------------------------------------------------------------
// Offscreen camera
// ------------------------------------------------------------------

/// Spawns a 2-D camera that renders to an offscreen `Image` asset instead
/// of a window surface.
///
/// The [`OffscreenRenderTarget`] resource is inserted so that other test
/// code (e.g. screenshot capture) can look up the image handle.
fn spawn_offscreen_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let width = 320;
    let height = 180;

    let size = Extent3d { width, height, depth_or_array_layers: 1 };

    let mut image = Image::new_fill(size, TextureDimension::D2, &[0, 0, 0, 255], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.insert_resource(OffscreenRenderTarget {
        handle: image_handle.clone(),
        width,
        height,
    });

    commands.spawn((Camera2d, Camera::default(), RenderTarget::Image(image_handle.into()), Transform::from_xyz(640.0, 360.0, 0.0)));
}
