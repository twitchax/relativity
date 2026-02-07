// Deny unwrap_used in production code to ensure proper error handling.
// Test code is allowed to use unwrap via #[cfg(test)] and module-level allows.
#![deny(unused)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::correctness)]
#![deny(clippy::complexity)]
#![deny(clippy::pedantic)]

pub mod game;
pub mod menu;
pub mod shared;

use bevy::prelude::*;
use game::{levels::CurrentLevel, GamePlugin};
use menu::MenuPlugin;
use shared::{state::AppState, types::spawn_camera};

fn main() {
    // Install better panic hook for WASM
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<CurrentLevel>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        }))
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .run();
}
