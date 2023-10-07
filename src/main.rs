pub mod game;
pub mod menu;
pub mod shared;

use bevy::prelude::*;
use game::{GamePlugin, levels::CurrentLevel};
use menu::MenuPlugin;
use shared::{state::AppState, types::spawn_camera};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .init_resource::<CurrentLevel>()
        .add_plugins(DefaultPlugins)
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .add_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .run();
}
