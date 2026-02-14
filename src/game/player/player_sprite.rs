use super::shared::Player;
use crate::{
    game::shared::{
        constants::MAX_PLAYER_LAUNCH_VELOCITY,
        types::{GameItem, LaunchState, Position, PowerBarUi, Radius, RocketSprite, TrailBuffer, Velocity},
    },
    shared::{state::GameState, SCREEN_WIDTH_PX},
};
use bevy::{prelude::*, window::PrimaryWindow};
use uom::si::f64::Velocity as UomVelocity;

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
    pub trail_buffer: TrailBuffer,
}

// Systems.

/// Phase 1: On mouse press, compute angle from player to cursor and lock the aim direction.
pub fn launch_aim_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut launch_state: ResMut<LaunchState>,
) {
    // Only transition from Idle on initial press.
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    if *launch_state != LaunchState::Idle {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_position): Option<Vec2> = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    let direction = cursor_world - player_pos;
    let angle = direction.y.atan2(direction.x);

    *launch_state = LaunchState::AimLocked { angle };
}

/// Phase 2: While the mouse is held, compute power from drag distance.
pub fn launch_power_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut launch_state: ResMut<LaunchState>,
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let (LaunchState::AimLocked { angle } | LaunchState::Launching { angle, .. }) = *launch_state else {
        return;
    };

    let Ok(player_transform) = player_query.single() else { return };
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_position): Option<Vec2> = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    let drag_distance = (cursor_world - player_pos).length();

    // Scale power: 0 at player, 1.0 at 80% of screen width.
    #[allow(clippy::cast_possible_truncation)]
    let max_drag = 0.8 * SCREEN_WIDTH_PX as f32;
    let power = (drag_distance / max_drag).min(1.0);

    *launch_state = LaunchState::Launching { angle, power };
}

/// Phase 3: On mouse release, fire the player with the locked angle and power.
pub fn launch_fire_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut player_velocity_query: Query<&mut Velocity, With<Player>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut launch_state: ResMut<LaunchState>,
) {
    if !mouse_input.just_released(MouseButton::Left) {
        return;
    }

    let LaunchState::Launching { angle, power } = *launch_state else {
        // If released from AimLocked (no drag), treat as cancel.
        if matches!(*launch_state, LaunchState::AimLocked { .. }) {
            *launch_state = LaunchState::Idle;
        }
        return;
    };

    let Ok(mut player_velocity) = player_velocity_query.single_mut() else { return };

    let (vx, vy) = calculate_launch_velocity_from_angle_power(angle, power, *MAX_PLAYER_LAUNCH_VELOCITY);

    player_velocity.x = vx;
    player_velocity.y = vy;

    *launch_state = LaunchState::Idle;
    game_state.set(GameState::Running);
}

/// Draw the aim direction line and power bar.
pub fn launch_visual_system(
    launch_state: Res<LaunchState>,
    player_query: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    power_bar_query: Query<Entity, With<PowerBarUi>>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    match *launch_state {
        LaunchState::AimLocked { angle } => {
            // Draw direction line.
            let direction = Vec2::new(angle.cos(), angle.sin());
            let line_end = player_pos + direction * 200.0;
            gizmos.line_2d(player_pos, line_end, Color::srgba(1.0, 1.0, 1.0, 0.7));

            // Despawn any leftover power bar.
            for entity in &power_bar_query {
                commands.entity(entity).despawn();
            }
        }
        LaunchState::Launching { angle, power } => {
            // Draw direction line, length scaled by power.
            let direction = Vec2::new(angle.cos(), angle.sin());
            let line_length = 100.0 + power * 200.0;
            let line_end = player_pos + direction * line_length;

            let color = Color::srgba(1.0, 1.0 - power, 0.0, 0.9);
            gizmos.line_2d(player_pos, line_end, color);

            // Spawn or update power bar UI.
            spawn_or_update_power_bar(&mut commands, &power_bar_query, power);
        }
        LaunchState::Idle => {
            // Despawn power bar when idle.
            for entity in &power_bar_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawn or replace the power-bar UI overlay.
fn spawn_or_update_power_bar(commands: &mut Commands, power_bar_query: &Query<Entity, With<PowerBarUi>>, power: f32) {
    // Despawn existing.
    for entity in power_bar_query {
        commands.entity(entity).despawn();
    }

    let bar_width = power * 200.0;

    commands
        .spawn((
            PowerBarUi,
            GameItem,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(50.0),
                width: Val::Px(204.0),
                height: Val::Px(24.0),
                margin: UiRect::left(Val::Px(-102.0)),
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(bar_width),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0 - power, 0.0, 0.9)),
            ));
        });
}

/// Computes launch velocity from angle and power (0.0–1.0).
///
/// Power is clamped to 0.99 to prevent reaching the speed of light.
#[must_use]
pub(crate) fn calculate_launch_velocity_from_angle_power(angle: f32, power: f32, max_velocity: UomVelocity) -> (UomVelocity, UomVelocity) {
    let clamped_power = power.min(0.99);
    let vx = max_velocity * f64::from(clamped_power) * f64::from(angle.cos());
    let vy = max_velocity * f64::from(clamped_power) * f64::from(angle.sin());
    (vx, vy)
}

/// Computes launch velocity from player position toward cursor position.
///
/// Power scales linearly with distance (clamped at 80% of screen width).
/// Retained for backward compatibility with existing tests.
#[cfg(test)]
pub(crate) fn calculate_launch_velocity(cursor_x: f64, cursor_y: f64, player_x: f64, player_y: f64, screen_width_px: f64, max_velocity: UomVelocity) -> (UomVelocity, UomVelocity) {
    use glam::DVec2;

    let launch_vector = DVec2::new(cursor_x - player_x, cursor_y - player_y);
    let launch_direction = launch_vector.normalize();
    let launch_power = f64::min(0.8 * screen_width_px, launch_vector.length()) / (0.8 * screen_width_px);

    (max_velocity * launch_power * launch_direction.x, max_velocity * launch_power * launch_direction.y)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::velocity::kilometer_per_second;

    const SCREEN_W: f64 = 1280.0;

    fn max_v() -> UomVelocity {
        UomVelocity::new::<kilometer_per_second>(100.0)
    }

    fn kps(v: UomVelocity) -> f64 {
        v.get::<kilometer_per_second>()
    }

    // --- calculate_launch_velocity (legacy pure function) ---

    #[test]
    fn direction_toward_cursor_right() {
        let (vx, vy) = calculate_launch_velocity(200.0, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        assert!(kps(vx) > 0.0, "vx should be positive (rightward)");
        assert_relative_eq!(kps(vy), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn direction_toward_cursor_left() {
        let (vx, vy) = calculate_launch_velocity(50.0, 100.0, 200.0, 100.0, SCREEN_W, max_v());
        assert!(kps(vx) < 0.0, "vx should be negative (leftward)");
        assert_relative_eq!(kps(vy), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn direction_toward_cursor_up() {
        let (vx, vy) = calculate_launch_velocity(100.0, 300.0, 100.0, 100.0, SCREEN_W, max_v());
        assert_relative_eq!(kps(vx), 0.0, epsilon = 1e-10);
        assert!(kps(vy) > 0.0, "vy should be positive (upward)");
    }

    #[test]
    fn direction_toward_cursor_down() {
        let (vx, vy) = calculate_launch_velocity(100.0, 50.0, 100.0, 200.0, SCREEN_W, max_v());
        assert_relative_eq!(kps(vx), 0.0, epsilon = 1e-10);
        assert!(kps(vy) < 0.0, "vy should be negative (downward)");
    }

    #[test]
    fn direction_diagonal() {
        let (vx, vy) = calculate_launch_velocity(200.0, 200.0, 100.0, 100.0, SCREEN_W, max_v());
        assert!(kps(vx) > 0.0);
        assert!(kps(vy) > 0.0);
        assert_relative_eq!(kps(vx), kps(vy), epsilon = 1e-10);
    }

    #[test]
    fn power_scales_with_distance() {
        let (vx_near, _) = calculate_launch_velocity(110.0, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        let (vx_far, _) = calculate_launch_velocity(500.0, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        assert!(kps(vx_far).abs() > kps(vx_near).abs(), "farther cursor should produce higher velocity");
    }

    #[test]
    fn power_clamps_at_max() {
        // Distance beyond 80% of screen width should clamp power at 1.0.
        let far_x = 100.0 + SCREEN_W * 2.0;
        let (vx, vy) = calculate_launch_velocity(far_x, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert_relative_eq!(speed, kps(max_v()), epsilon = 1e-6);
    }

    #[test]
    fn at_exactly_80_percent_screen_width() {
        let dist = 0.8 * SCREEN_W;
        let (vx, vy) = calculate_launch_velocity(100.0 + dist, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert_relative_eq!(speed, kps(max_v()), epsilon = 1e-6);
    }

    #[test]
    fn respects_max_velocity() {
        // Even at maximum power, speed should not exceed max_velocity.
        let (vx, vy) = calculate_launch_velocity(5000.0, 5000.0, 0.0, 0.0, SCREEN_W, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert!(speed <= kps(max_v()) + 1e-6, "speed should not exceed max velocity");
    }

    #[test]
    fn very_short_distance_produces_small_velocity() {
        let (vx, vy) = calculate_launch_velocity(100.001, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert!(speed < kps(max_v()) * 0.01, "very short distance should produce very small velocity");
    }

    #[test]
    fn different_max_velocity() {
        let small_max = UomVelocity::new::<kilometer_per_second>(10.0);
        let (vx, vy) = calculate_launch_velocity(5000.0, 100.0, 100.0, 100.0, SCREEN_W, small_max);
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert_relative_eq!(speed, kps(small_max), epsilon = 1e-6);
    }

    #[test]
    fn symmetric_horizontal() {
        let (vx_right, _) = calculate_launch_velocity(200.0, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        let (vx_left, _) = calculate_launch_velocity(0.0, 100.0, 100.0, 100.0, SCREEN_W, max_v());
        assert_relative_eq!(kps(vx_right).abs(), kps(vx_left).abs(), epsilon = 1e-10);
    }

    // --- calculate_launch_velocity_from_angle_power ---

    #[test]
    fn angle_power_right_full_power() {
        let (vx, vy) = calculate_launch_velocity_from_angle_power(0.0, 0.99, max_v());
        assert!(kps(vx) > 0.0, "vx should be positive (rightward)");
        assert_relative_eq!(kps(vy), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn angle_power_up_full_power() {
        let angle = std::f32::consts::FRAC_PI_2;
        let (vx, vy) = calculate_launch_velocity_from_angle_power(angle, 0.99, max_v());
        assert_relative_eq!(kps(vx), 0.0, epsilon = 1e-2);
        assert!(kps(vy) > 0.0, "vy should be positive (upward)");
    }

    #[test]
    fn angle_power_zero_power_produces_zero_velocity() {
        let (vx, vy) = calculate_launch_velocity_from_angle_power(0.5, 0.0, max_v());
        assert_relative_eq!(kps(vx), 0.0, epsilon = 1e-10);
        assert_relative_eq!(kps(vy), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn angle_power_clamped_at_ninety_nine_percent() {
        let (vx, vy) = calculate_launch_velocity_from_angle_power(0.0, 1.5, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert_relative_eq!(speed, kps(max_v()) * 0.99, epsilon = 1e-4);
    }

    #[test]
    fn angle_power_half_power_is_proportional() {
        let result_half = calculate_launch_velocity_from_angle_power(0.0, 0.5, max_v());
        let result_full = calculate_launch_velocity_from_angle_power(0.0, 0.99, max_v());
        let speed_half = (kps(result_half.0).powi(2) + kps(result_half.1).powi(2)).sqrt();
        let speed_full = (kps(result_full.0).powi(2) + kps(result_full.1).powi(2)).sqrt();
        assert_relative_eq!(speed_half / speed_full, 0.5 / 0.99, epsilon = 1e-4);
    }

    #[test]
    fn angle_power_diagonal() {
        let angle = std::f32::consts::FRAC_PI_4;
        let (vx, vy) = calculate_launch_velocity_from_angle_power(angle, 0.5, max_v());
        assert!(kps(vx) > 0.0);
        assert!(kps(vy) > 0.0);
        assert_relative_eq!(kps(vx), kps(vy), epsilon = 1e-4);
    }

    // --- LaunchState ---

    #[test]
    fn launch_state_default_is_idle() {
        assert_eq!(LaunchState::default(), LaunchState::Idle);
    }

    #[test]
    fn launch_state_variants_are_distinct() {
        let idle = LaunchState::Idle;
        let aim = LaunchState::AimLocked { angle: 0.0 };
        let launch = LaunchState::Launching { angle: 0.0, power: 0.5 };
        assert_ne!(idle, aim);
        assert_ne!(idle, launch);
        assert_ne!(aim, launch);
    }
}
