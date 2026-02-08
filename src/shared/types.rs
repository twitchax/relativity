use bevy::prelude::*;
use bevy_lunex::prelude::UiSourceCamera;
use bevy_trauma_shake::Shake;

use super::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

// Camera.

#[allow(clippy::cast_possible_truncation)]
pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(SCREEN_WIDTH_PX as f32 / 2.0, SCREEN_HEIGHT_PX as f32 / 2.0, 0.0);

    commands.spawn((Camera2d, transform, Shake::default(), UiSourceCamera::<0>));
}
