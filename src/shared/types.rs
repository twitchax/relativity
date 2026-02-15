use bevy::camera::ScalingMode;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_lunex::prelude::UiSourceCamera;
use bevy_trauma_shake::Shake;

use super::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

// Camera.

#[allow(clippy::cast_possible_truncation)]
pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(SCREEN_WIDTH_PX as f32 / 2.0, SCREEN_HEIGHT_PX as f32 / 2.0, 0.0);

    let projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: SCREEN_WIDTH_PX as f32,
            height: SCREEN_HEIGHT_PX as f32,
        },
        ..OrthographicProjection::default_2d()
    });

    // Explicit Tonemapping::None avoids needing the `tonemapping_luts` feature,
    // which embeds ~15-20 MB of LUT data into the binary.
    commands.spawn((Camera2d, projection, transform, Tonemapping::None, Shake::default(), UiSourceCamera::<0>));
}
