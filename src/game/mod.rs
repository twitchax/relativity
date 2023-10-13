pub mod destination;
pub mod levels;
pub mod object;
pub mod observer;
pub mod player;
pub mod shared;

use bevy::prelude::*;

use crate::shared::state::{AppState, GameState};

use self::{
    levels::{despawn_level, spawn_level},
    observer::{observer_clock_text_update, observer_clock_update},
    player::{
        player_clock::{player_clock_text_update, player_clock_update},
        player_sprite::player_launch,
    },
    shared::systems::{
        exit_level_check, planet_scale_update, position_update, rocket_rotation_update, rocket_scale_update, translation_update,
        velocity_update, collision_check,
    },
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            // Spawn things on enter.
            .add_systems(OnEnter(AppState::InGame), spawn_level)
            // Destroy things on exit.
            .add_systems(OnExit(AppState::InGame), despawn_level)
            // Run the scale updates always.
            .add_systems(
                Update,
                (planet_scale_update, rocket_scale_update, exit_level_check).run_if(in_state(AppState::InGame)),
            )
            // Allow launching if paused.
            .add_systems(
                Update,
                (player_launch, translation_update)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(GameState::Paused)),
            )
            // Run the rest of the updates if running.
            .add_systems(
                Update,
                (
                    rocket_rotation_update,
                    velocity_update,
                    position_update.after(velocity_update),
                    translation_update.after(position_update),
                    collision_check,
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
