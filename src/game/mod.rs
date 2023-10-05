pub mod shared;
pub mod player;
pub mod object;
pub mod observer;
pub mod destination;

use bevy::prelude::*;

use crate::shared::state::{GameState, AppState};

use self::{shared::systems::{spawn_camera, scale_update, translation_update, velocity_update, position_update}, observer::{spawn_observer, observer_clock_update, observer_clock_text_update}, player::{player_sprite::{spawn_player_sprite, player_launch}, player_clock::{player_clock_update, player_clock_text_update}}, object::spawn_planets};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
            .add_systems(OnEnter(AppState::InGame), (spawn_camera, spawn_observer, spawn_player_sprite, spawn_planets))
            .add_systems(Update, scale_update.run_if(in_state(AppState::InGame)))
            .add_systems(Update, (player_launch, translation_update).run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Paused)))
            .add_systems(
                Update,
                (
                    velocity_update,
                    position_update.after(velocity_update),
                    translation_update.after(position_update),
                    observer_clock_update,
                    observer_clock_text_update.after(observer_clock_update),
                    player_clock_update,
                    player_clock_text_update.after(player_clock_update),
                ).run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Running)),
            );
    }
}