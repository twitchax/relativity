pub mod destination;
pub mod object;
pub mod observer;
pub mod player;
pub mod shared;

use bevy::prelude::*;

use crate::shared::state::{AppState, GameState};

use self::{
    object::spawn_planets,
    observer::{observer_clock_text_update, observer_clock_update, spawn_observer_clock},
    player::{
        player_clock::{player_clock_text_update, player_clock_update, spawn_player_clock},
        player_sprite::{player_launch, spawn_player_sprite},
    },
    shared::systems::{position_update, scale_update, spawn_camera, translation_update, velocity_update},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    spawn_camera,
                    spawn_observer_clock,
                    spawn_player_sprite,
                    spawn_player_clock,
                    spawn_planets,
                ),
            )
            .add_systems(Update, scale_update.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                (player_launch, translation_update)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::Paused)),
            )
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
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::Running)),
            );
    }
}
