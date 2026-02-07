// Headless render smoke test: verifies the game bootstraps and the render graph
// executes without a display server or GPU window.
//
// Uses DefaultPlugins with WinitPlugin disabled and no primary window.
// A camera renders to an offscreen Image target so the render graph actually
// executes. We drive the app manually via `app.update()` instead of `app.run()`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

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

/// Build the app the same way `main()` does, but headless:
/// - `DefaultPlugins` with `WinitPlugin` disabled and no primary window
/// - Game and menu plugins
/// - An offscreen camera rendering to an `Image` asset
fn build_headless_app() -> App {
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

/// Spawns a 2-D camera that renders to an offscreen `Image` asset instead of a
/// window surface.  This lets the render graph execute without requiring an OS
/// window or display server.
fn spawn_offscreen_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = Extent3d {
        width: 320,
        height: 180,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(size, TextureDimension::D2, &[0, 0, 0, 255], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Image(image_handle.into()),
            ..default()
        },
        Transform::from_xyz(640.0, 360.0, 0.0),
    ));
}

#[test]
fn headless_app_boots_and_runs_without_panic() {
    let mut app = build_headless_app();

    // Complete plugin initialization before calling update().
    // The render plugin initializes the GPU asynchronously; finish() and
    // cleanup() ensure all plugin lifecycle hooks have run.
    while app.plugins_state() != bevy::app::PluginsState::Cleaned {
        if app.plugins_state() == bevy::app::PluginsState::Ready {
            app.finish();
            app.cleanup();
        }
    }

    // Run several update cycles — the primary goal is to prove the app
    // bootstraps and the render graph executes without panicking.
    for _ in 0..5 {
        app.update();
    }
}
