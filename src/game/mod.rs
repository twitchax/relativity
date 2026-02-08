pub mod destination;
pub mod gravity_grid;
pub mod levels;
pub mod object;
pub mod observer;
pub mod outcome;
pub mod player;
pub mod shared;
pub mod trail;

#[cfg(test)]
pub mod test_helpers;

use bevy::prelude::*;

use crate::shared::state::{AppState, GameState};

use self::{
    gravity_grid::gravity_grid_render_system,
    levels::{despawn_level, spawn_level},
    observer::{observer_clock_text_update, observer_clock_update},
    outcome::{despawn_failure_overlay, despawn_success_overlay, failure_auto_reset, spawn_failure_overlay, spawn_success_overlay, success_button_interaction},
    player::{
        player_clock::{player_clock_text_update, player_clock_update},
        player_sprite::{launch_aim_system, launch_fire_system, launch_power_system, launch_visual_system},
    },
    shared::{
        systems::{collision_check, exit_level_check, planet_scale_update, position_update, rocket_rotation_update, rocket_scale_update, translation_update, velocity_update},
        types::LaunchState,
    },
    trail::{trail_clear_system, trail_record_system, trail_render_system},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<LaunchState>()
            // Spawn things on enter.
            .add_systems(OnEnter(AppState::InGame), spawn_level)
            // Destroy things on exit.
            .add_systems(OnExit(AppState::InGame), despawn_level)
            // Success overlay lifecycle.
            .add_systems(OnEnter(GameState::Finished), spawn_success_overlay)
            .add_systems(OnExit(GameState::Finished), despawn_success_overlay)
            // Failure overlay lifecycle.
            .add_systems(OnEnter(GameState::Failed), spawn_failure_overlay)
            .add_systems(OnExit(GameState::Failed), despawn_failure_overlay)
            // Run the scale updates always.
            .add_systems(Update, (planet_scale_update, rocket_scale_update, exit_level_check).run_if(in_state(AppState::InGame)))
            // Success overlay button interaction while finished.
            .add_systems(Update, success_button_interaction.run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Finished)))
            // Failure auto-reset timer while failed.
            .add_systems(Update, failure_auto_reset.run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Failed)))
            // Clear trail buffer on level reset.
            .add_systems(OnEnter(GameState::Paused), trail_clear_system)
            // Render trail and gravity grid while in game (visible across all sub-states).
            .add_systems(Update, (trail_render_system, gravity_grid_render_system).run_if(in_state(AppState::InGame)))
            // Launch mechanic (aim, power, fire, visuals) while paused.
            .add_systems(
                Update,
                (launch_aim_system, launch_power_system, launch_fire_system, launch_visual_system, translation_update)
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
                    trail_record_system.after(player_clock_update),
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
