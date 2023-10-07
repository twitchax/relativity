use bevy::prelude::*;

use super::{SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX};

// Camera.

pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(SCREEN_WIDTH_PX as f32 / 2.0, SCREEN_HEIGHT_PX as f32 / 2.0, 0.0);

    commands.spawn(Camera2dBundle { transform, ..Default::default() });
}