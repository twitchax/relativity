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
}

// Systems.

#[allow(clippy::needless_pass_by_value)]
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
    let cursor_transform = DVec2::new(f64::from(cursor_position.x), SCREEN_HEIGHT_PX - f64::from(cursor_position.y));

    let (vx, vy) = calculate_launch_velocity(
        cursor_transform.x,
        cursor_transform.y,
        f64::from(player_transform.translation.x),
        f64::from(player_transform.translation.y),
        SCREEN_WIDTH_PX,
        *MAX_PLAYER_LAUNCH_VELOCITY,
    );

    player_velocity.x = vx;
    player_velocity.y = vy;

    state.set(GameState::Running);
}

/// Computes launch velocity from player position toward cursor position.
///
/// Power scales linearly with distance (clamped at 80% of screen width).
pub(crate) fn calculate_launch_velocity(cursor_x: f64, cursor_y: f64, player_x: f64, player_y: f64, screen_width_px: f64, max_velocity: UomVelocity) -> (UomVelocity, UomVelocity) {
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
}
