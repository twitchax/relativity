pub mod shared;
pub mod game;
pub mod menu;

use bevy::prelude::*;
use game::GamePlugin;
use menu::MenuPlugin;
use shared::state::AppState;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .add_state::<AppState>()
        .run();
}