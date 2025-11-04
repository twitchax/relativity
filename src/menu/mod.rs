use crate::shared::state::AppState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, start.run_if(in_state(AppState::Menu)));
    }
}

pub fn start(mut mouse_input: ResMut<ButtonInput<MouseButton>>, mut state: ResMut<NextState<AppState>>) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    mouse_input.release_all();
    mouse_input.reset_all();

    state.set(AppState::InGame);
}
