use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_lunex::prelude::UiLunexPlugins;
use bevy_trauma_shake::TraumaPlugin;
use relativity::{
    game::{levels::CurrentLevel, GamePlugin},
    menu::MenuPlugin,
    shared::{state::AppState, types::spawn_camera},
};

fn main() {
    // Install better panic hook for WASM
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<CurrentLevel>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Configure the canvas selector for WASM
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#bevy-canvas".to_string()),
                        // Fit canvas to parent container for better web experience
                        #[cfg(target_arch = "wasm32")]
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                // In WASM, Trunk's SPA fallback returns index.html for missing .meta files,
                // which Bevy then fails to parse as RON. Skip .meta file checks entirely.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(TraumaPlugin)
        .add_plugins(UiLunexPlugins)
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .run();
}
