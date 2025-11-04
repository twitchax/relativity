use super::shared::Player;
use crate::{
    game::shared::{
        constants::MAX_PLAYER_LAUNCH_VELOCITY,
        types::{GameItem, Position, Radius, RocketSprite, Velocity},
    },
    shared::{state::GameState, SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX},
};
use bevy::{prelude::*, window::PrimaryWindow};
use glam::DVec2;

// Components / bundles.

#[derive(Bundle, Default)]
pub struct PlayerSpriteBundle {
    pub item: GameItem,
    pub player: Player,
    pub position: Position,
    pub radius: Radius,
    pub velocity: Velocity,
    pub sprite_type: RocketSprite,
    pub sprite: Sprite,
    pub transform: Transform,
}

// Systems.

pub fn player_launch(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut player_velocity_query: Query<(&Transform, &mut Velocity), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<NextState<GameState>>,
) {
    let Ok((player_transform, mut player_velocity)) = player_velocity_query.single_mut() else {
        return;
    };

    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.single() else { return };
    let Some(cursor_position) = window.cursor_position() else { return };
    let cursor_transform = DVec2::new(cursor_position.x as f64, SCREEN_HEIGHT_PX - cursor_position.y as f64);

    let launch_vector = DVec2::new(
        cursor_transform.x - player_transform.translation.x as f64,
        cursor_transform.y - player_transform.translation.y as f64,
    );
    let launch_direction = launch_vector.normalize();
    let launch_power = f64::min(0.8 * SCREEN_WIDTH_PX, launch_vector.length()) / (0.8 * SCREEN_WIDTH_PX);

    player_velocity.x = *MAX_PLAYER_LAUNCH_VELOCITY * launch_power * launch_direction.x;
    player_velocity.y = *MAX_PLAYER_LAUNCH_VELOCITY * launch_power * launch_direction.y;

    state.set(GameState::Running);
}
