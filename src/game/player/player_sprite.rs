use super::shared::Player;
use crate::{
    game::shared::{
        constants::MAX_PLAYER_LAUNCH_VELOCITY,
        types::{GameItem, LaunchState, Position, Radius, RocketSprite, TrailBuffer, Velocity, VelocityReadout},
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

// Constants.

/// Length of the dotted aim-preview line (pixels).
const PREVIEW_LINE_LENGTH: f32 = 300.0;
/// Length of each dash segment in the preview line (pixels).
const DASH_LENGTH: f32 = 10.0;
/// Gap between dash segments in the preview line (pixels).
const DASH_GAP: f32 = 8.0;

/// Solid aim-line length in `AimLocked` state (pixels).
const AIM_LINE_LENGTH: f32 = 200.0;
/// Minimum solid direction-line length in `Launching` state (pixels).
const MIN_LAUNCH_LINE: f32 = 100.0;
/// Maximum solid direction-line length at full power (pixels).
const MAX_LAUNCH_LINE: f32 = 300.0;

/// Radius of the radial power arc around the player (pixels).
const ARC_RADIUS: f32 = 50.0;
/// Maximum sweep angle for the power arc (270°).
const MAX_ARC_ANGLE: f32 = std::f32::consts::FRAC_PI_2 * 3.0;

/// Half-length of each tick mark (extends inward and outward from arc radius).
const TICK_HALF_LENGTH: f32 = 6.0;
/// Velocity fractions (of c) at which tick marks are drawn on the arc.
const TICK_VELOCITY_FRACTIONS: [f32; 4] = [0.25, 0.5, 0.75, 0.9];

/// Minimum velocity fraction (of `max_velocity`) at zero raw power.
const MIN_POWER_FRACTION: f32 = 0.1 / 0.99;

/// Offset beyond the arc radius at which the velocity readout text is placed.
const READOUT_OFFSET: f32 = 18.0;
/// Font size of the velocity readout text entity.
const READOUT_FONT_SIZE: f32 = 14.0;

// Helpers.

/// Maps raw drag power (0.0–1.0) to an effective power fraction (≈0.101–1.0)
/// using a quadratic ease-in curve with a 0.1c minimum velocity floor.
///
/// Result × `max_velocity` (0.99c) gives the launched velocity:
/// - raw 0.0 → ≈0.101 → 0.1c
/// - raw 1.0 → 1.0   → 0.99c
pub(crate) fn map_power_nonlinear(raw_power: f32) -> f32 {
    let p = raw_power.clamp(0.0, 1.0);
    MIN_POWER_FRACTION + (1.0 - MIN_POWER_FRACTION) * p * p
}

/// Draws tick marks on the radial arc at predefined velocity fractions.
///
/// Each tick is a short radial line spanning `ARC_RADIUS ± TICK_HALF_LENGTH`.
/// The angular position is derived from the velocity fraction mapped to the
/// arc's sweep range.
fn draw_arc_ticks(gizmos: &mut Gizmos, center: Vec2, arc_rotation_rad: f32) {
    for &frac in &TICK_VELOCITY_FRACTIONS {
        // Mapped power corresponding to this velocity fraction.
        let tick_power = frac / 0.99;
        // Position along the arc sweep: fraction of MAX_ARC_ANGLE, offset from centre.
        let local_angle = MAX_ARC_ANGLE * (tick_power - 0.5);
        let world_angle = arc_rotation_rad + local_angle;
        let radial = Vec2::new(world_angle.cos(), world_angle.sin());

        let inner = center + radial * (ARC_RADIUS - TICK_HALF_LENGTH);
        let outer = center + radial * (ARC_RADIUS + TICK_HALF_LENGTH);
        gizmos.line_2d(inner, outer, Color::srgba(1.0, 1.0, 1.0, 0.5));
    }
}

/// Draws a dashed line from `start` along `direction` for `length` pixels.
fn draw_dashed_line(gizmos: &mut Gizmos, start: Vec2, direction: Vec2, length: f32, color: Color) {
    let stride = DASH_LENGTH + DASH_GAP;
    let mut offset = 0.0;

    while offset < length {
        let dash_end = (offset + DASH_LENGTH).min(length);
        let s = start + direction * offset;
        let e = start + direction * dash_end;
        gizmos.line_2d(s, e, color);
        offset += stride;
    }
}

// Systems.

/// Draws a dotted aim-preview line from the player toward the cursor while idle.
///
/// Runs every frame during `GameState::Paused` when `LaunchState::Idle`.
/// The line is rendered as a series of short dash segments via `Gizmos`.
pub fn launch_preview_system(
    launch_state: Res<LaunchState>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut gizmos: Gizmos,
) {
    if *launch_state != LaunchState::Idle {
        return;
    }

    let Ok(player_transform) = player_query.single() else { return };
    let Ok(window) = window_query.single() else { return };
    let Some(cursor_position) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let direction = (cursor_world - player_pos).normalize_or_zero();

    if direction == Vec2::ZERO {
        return;
    }

    draw_dashed_line(&mut gizmos, player_pos, direction, PREVIEW_LINE_LENGTH, Color::srgba(1.0, 1.0, 1.0, 0.3));
}

/// Cancels the launch on right-click or Escape from any non-Idle state.
pub fn launch_cancel_system(mouse_input: Res<ButtonInput<MouseButton>>, keyboard_input: Res<ButtonInput<KeyCode>>, mut launch_state: ResMut<LaunchState>) {
    if *launch_state == LaunchState::Idle {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Right) || keyboard_input.just_pressed(KeyCode::Escape) {
        *launch_state = LaunchState::Idle;
    }
}

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

/// Maps power (0.0–1.0) to a color gradient: cyan → orange → red.
fn power_to_color(power: f32) -> Color {
    let t = power.clamp(0.0, 1.0);

    let (r, g, b) = if t < 0.5 {
        let f = t * 2.0;
        (f, 1.0 - 0.35 * f, 1.0 - f)
    } else {
        let f = (t - 0.5) * 2.0;
        (1.0, 0.65 * (1.0 - f), 0.0)
    };

    Color::srgba(r, g, b, 0.9)
}

/// Draw the aim direction line and radial power arc around the player.
///
/// - **`AimLocked`**: solid direction line + dotted extension + faint arc outline.
/// - **`Launching`**: solid direction line (length scaled by power) + dotted
///   extension to max range + filled arc with color gradient.
pub fn launch_visual_system(launch_state: Res<LaunchState>, player_query: Query<&Transform, With<Player>>, mut gizmos: Gizmos) {
    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    // Rotate so the arc gap sits at the bottom.
    let arc_rotation = Rot2::radians(-3.0 * std::f32::consts::FRAC_PI_4);

    match *launch_state {
        LaunchState::AimLocked { angle } => {
            let direction = Vec2::new(angle.cos(), angle.sin());

            // Solid direction line.
            let line_end = player_pos + direction * AIM_LINE_LENGTH;
            gizmos.line_2d(player_pos, line_end, Color::srgba(1.0, 1.0, 1.0, 0.7));

            // Dotted extension beyond the solid line.
            let extension_length = MAX_LAUNCH_LINE - AIM_LINE_LENGTH;
            draw_dashed_line(&mut gizmos, line_end, direction, extension_length, Color::srgba(1.0, 1.0, 1.0, 0.15));

            // Faint arc outline showing max range.
            let isometry = Isometry2d::new(player_pos, arc_rotation);
            gizmos.arc_2d(isometry, MAX_ARC_ANGLE, ARC_RADIUS, Color::srgba(1.0, 1.0, 1.0, 0.15));
        }
        LaunchState::Launching { angle, power } => {
            let direction = Vec2::new(angle.cos(), angle.sin());
            let mapped_power = map_power_nonlinear(power);

            // Solid direction line, length scaled by mapped power.
            let line_length = MIN_LAUNCH_LINE + mapped_power * (MAX_LAUNCH_LINE - MIN_LAUNCH_LINE);
            let line_end = player_pos + direction * line_length;

            let color = power_to_color(mapped_power);
            gizmos.line_2d(player_pos, line_end, color);

            // Dotted extension beyond the power-scaled line.
            let extension_length = MAX_LAUNCH_LINE - line_length;
            if extension_length > 0.0 {
                draw_dashed_line(&mut gizmos, line_end, direction, extension_length, Color::srgba(1.0, 1.0, 1.0, 0.1));
            }

            // Faint arc outline showing max range.
            let outline_iso = Isometry2d::new(player_pos, arc_rotation);
            gizmos.arc_2d(outline_iso, MAX_ARC_ANGLE, ARC_RADIUS, Color::srgba(1.0, 1.0, 1.0, 0.15));

            // Filled arc proportional to mapped power.
            let filled_angle = mapped_power * MAX_ARC_ANGLE;
            let filled_iso = Isometry2d::new(player_pos, arc_rotation);
            gizmos.arc_2d(filled_iso, filled_angle, ARC_RADIUS, color);

            // Tick marks at notable velocity fractions.
            let arc_rotation_rad = -3.0 * std::f32::consts::FRAC_PI_4;
            draw_arc_ticks(&mut gizmos, player_pos, arc_rotation_rad);
        }
        LaunchState::Idle => {}
    }
}

/// Manages the velocity readout text entity near the radial arc.
///
/// Spawns a `Text2d` with `VelocityReadout` marker when `Launching`, updates its
/// content and position each frame, and despawns it in any other state.
#[allow(clippy::type_complexity)]
pub fn launch_readout_system(
    launch_state: Res<LaunchState>,
    player_query: Query<&Transform, With<Player>>,
    mut readout_query: Query<(Entity, &mut Text2d, &mut Transform), (With<VelocityReadout>, Without<Player>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let LaunchState::Launching { power, .. } = *launch_state else {
        // Despawn readout when not launching.
        for (entity, _, _) in &readout_query {
            commands.entity(entity).despawn();
        }
        return;
    };

    let Ok(player_transform) = player_query.single() else { return };
    let player_pos = player_transform.translation.truncate();

    let velocity_fraction = map_power_nonlinear(power) * 0.99;
    let text = format!("{velocity_fraction:.2}c");

    // Position the readout at the tip of the filled arc, just outside.
    let arc_rotation_rad = -3.0 * std::f32::consts::FRAC_PI_4;
    let filled_angle = map_power_nonlinear(power) * MAX_ARC_ANGLE;
    let tip_angle = arc_rotation_rad + filled_angle * 0.5;
    let readout_pos = player_pos + Vec2::new(tip_angle.cos(), tip_angle.sin()) * (ARC_RADIUS + READOUT_OFFSET);

    if let Ok((_, mut text_component, mut transform)) = readout_query.single_mut() {
        // Update existing readout.
        *text_component = Text2d::new(text);
        transform.translation = readout_pos.extend(10.0);
    } else {
        // Spawn new readout entity.
        let color = power_to_color(map_power_nonlinear(power));
        commands.spawn((
            VelocityReadout,
            Text2d::new(text),
            TextFont {
                font: asset_server.load("fonts/Orbitron-Regular.ttf"),
                font_size: READOUT_FONT_SIZE,
                ..Default::default()
            },
            TextColor(color),
            Transform::from_translation(readout_pos.extend(10.0)),
        ));
    }
}

/// Computes launch velocity from angle and raw power (0.0–1.0).
///
/// Applies a non-linear (quadratic ease-in) power curve mapping raw power to
/// a velocity fraction of 0.1c–0.99c, giving fine control at low speeds.
#[must_use]
pub(crate) fn calculate_launch_velocity_from_angle_power(angle: f32, power: f32, max_velocity: UomVelocity) -> (UomVelocity, UomVelocity) {
    let mapped = f64::from(map_power_nonlinear(power));
    let vx = max_velocity * mapped * f64::from(angle.cos());
    let vy = max_velocity * mapped * f64::from(angle.sin());
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
    fn angle_power_zero_power_produces_minimum_velocity() {
        let (vx, vy) = calculate_launch_velocity_from_angle_power(0.0, 0.0, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        // Zero raw power should produce the minimum velocity (≈0.1c fraction of max_v).
        let expected = kps(max_v()) * f64::from(MIN_POWER_FRACTION);
        assert_relative_eq!(speed, expected, epsilon = 1e-2);
    }

    #[test]
    fn angle_power_clamped_at_max() {
        // Power > 1.0 is clamped to 1.0, giving full mapped velocity.
        let (vx, vy) = calculate_launch_velocity_from_angle_power(0.0, 1.5, max_v());
        let speed = (kps(vx).powi(2) + kps(vy).powi(2)).sqrt();
        assert_relative_eq!(speed, kps(max_v()), epsilon = 1e-2);
    }

    #[test]
    fn angle_power_nonlinear_half_power_slower_than_linear() {
        // With quadratic ease-in, half raw power gives less than half max speed.
        let result_half = calculate_launch_velocity_from_angle_power(0.0, 0.5, max_v());
        let speed_half = (kps(result_half.0).powi(2) + kps(result_half.1).powi(2)).sqrt();
        let mapped_half = f64::from(map_power_nonlinear(0.5));
        assert_relative_eq!(speed_half, kps(max_v()) * mapped_half, epsilon = 1e-2);
        // Verify non-linearity: mapped 0.5 < 0.5 (quadratic ease-in is slower early).
        assert!(mapped_half < 0.5, "quadratic ease-in should give less than half at raw_power=0.5");
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
