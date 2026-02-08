use bevy::prelude::*;

use crate::game::{
    player::shared::Player,
    shared::{
        helpers::get_translation_from_position,
        types::{GravitationalGamma, Position, TrailBuffer, VelocityGamma},
    },
};

/// Maximum number of trail points stored in the buffer.
const MAX_TRAIL_POINTS: usize = 2000;

// Pure functions.

/// Map a combined gamma value to a color.
///
/// - γ ≈ 1.0 → blue/cyan (cool)
/// - γ > 2.0 → red/orange (warm)
///
/// Interpolates linearly between cold and hot colors.
#[must_use]
pub(crate) fn gamma_to_color(total_gamma: f64) -> Color {
    // Normalize gamma to a 0–1 range: γ=1 → t=0, γ≥3 → t=1.
    #[allow(clippy::cast_possible_truncation)]
    let blend = ((total_gamma - 1.0) / 2.0).clamp(0.0, 1.0) as f32;

    // Cold: blue/cyan (0.2, 0.6, 1.0) → Hot: red/orange (1.0, 0.3, 0.0).
    let red = 0.2 + blend * 0.8;
    let green = 0.6 - blend * 0.3;
    let blue = 1.0 - blend;
    let alpha = 0.7 + blend * 0.2;

    Color::srgba(red, green, blue, alpha)
}

// Systems.

/// Each frame during Running, record the player's screen position and a gamma-derived color.
pub fn trail_record_system(
    player_query: Query<&Position, With<Player>>,
    gamma_query: Query<(&VelocityGamma, &GravitationalGamma), With<Player>>,
    mut trail_query: Query<&mut TrailBuffer, With<Player>>,
) {
    let Ok(position) = player_query.single() else { return };
    let Ok((velocity_gamma, gravitational_gamma)) = gamma_query.single() else { return };
    let Ok(mut trail) = trail_query.single_mut() else { return };

    let screen_pos = get_translation_from_position(position).truncate();
    let total_gamma = velocity_gamma.value * gravitational_gamma.value;
    let color = gamma_to_color(total_gamma);

    trail.points.push((screen_pos, color));

    // Cap buffer length.
    if trail.points.len() > MAX_TRAIL_POINTS {
        let excess = trail.points.len() - MAX_TRAIL_POINTS;
        trail.points.drain(..excess);
    }
}

/// Render the trail buffer using a gradient line strip.
pub fn trail_render_system(trail_query: Query<&TrailBuffer, With<Player>>, mut gizmos: Gizmos) {
    let Ok(trail) = trail_query.single() else { return };

    if trail.points.len() < 2 {
        return;
    }

    gizmos.linestrip_gradient_2d(trail.points.iter().copied());
}

/// Clear the trail buffer when re-entering the Paused state (level reset).
pub fn trail_clear_system(mut trail_query: Query<&mut TrailBuffer, With<Player>>) {
    let Ok(mut trail) = trail_query.single_mut() else { return };
    trail.points.clear();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // --- gamma_to_color ---

    #[test]
    fn gamma_one_produces_cool_color() {
        let color = gamma_to_color(1.0);
        let srgba = color.to_srgba();
        // Should be blue/cyan: low red, high blue.
        assert!(srgba.red < 0.4, "red should be low at γ=1, got {}", srgba.red);
        assert!(srgba.blue > 0.8, "blue should be high at γ=1, got {}", srgba.blue);
    }

    #[test]
    fn gamma_three_or_higher_produces_warm_color() {
        let color = gamma_to_color(3.0);
        let srgba = color.to_srgba();
        // Should be red/orange: high red, low blue.
        assert!(srgba.red > 0.8, "red should be high at γ=3, got {}", srgba.red);
        assert!(srgba.blue < 0.2, "blue should be low at γ=3, got {}", srgba.blue);
    }

    #[test]
    fn gamma_two_produces_intermediate_color() {
        let color = gamma_to_color(2.0);
        let srgba = color.to_srgba();
        // Midpoint: r ≈ 0.6, g ≈ 0.45, b ≈ 0.5.
        assert!(srgba.red > 0.4 && srgba.red < 0.8, "red should be intermediate at γ=2, got {}", srgba.red);
        assert!(srgba.blue > 0.3 && srgba.blue < 0.7, "blue should be intermediate at γ=2, got {}", srgba.blue);
    }

    #[test]
    fn gamma_below_one_clamps_to_cool() {
        let color = gamma_to_color(0.5);
        let srgba = color.to_srgba();
        let color_one = gamma_to_color(1.0);
        let srgba_one = color_one.to_srgba();
        // Below 1.0 should clamp to the same as 1.0.
        approx::assert_relative_eq!(srgba.red, srgba_one.red, epsilon = 1e-5);
        approx::assert_relative_eq!(srgba.blue, srgba_one.blue, epsilon = 1e-5);
    }

    #[test]
    fn gamma_very_high_clamps_to_warm() {
        let color = gamma_to_color(100.0);
        let srgba = color.to_srgba();
        let color_three = gamma_to_color(3.0);
        let srgba_three = color_three.to_srgba();
        // Very high gamma should clamp to the same as 3.0.
        approx::assert_relative_eq!(srgba.red, srgba_three.red, epsilon = 1e-5);
        approx::assert_relative_eq!(srgba.blue, srgba_three.blue, epsilon = 1e-5);
    }

    #[test]
    fn gamma_color_alpha_increases_with_gamma() {
        let alpha_low = gamma_to_color(1.0).to_srgba().alpha;
        let alpha_high = gamma_to_color(3.0).to_srgba().alpha;
        assert!(alpha_high > alpha_low, "alpha should increase with gamma");
    }

    #[test]
    fn gamma_color_is_always_valid() {
        for gamma in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0, 10.0] {
            let color = gamma_to_color(gamma);
            let srgba = color.to_srgba();
            assert!((0.0..=1.0).contains(&srgba.red), "red out of range for γ={gamma}");
            assert!((0.0..=1.0).contains(&srgba.green), "green out of range for γ={gamma}");
            assert!((0.0..=1.0).contains(&srgba.blue), "blue out of range for γ={gamma}");
            assert!((0.0..=1.0).contains(&srgba.alpha), "alpha out of range for γ={gamma}");
        }
    }

    // --- TrailBuffer ---

    #[test]
    fn trail_buffer_default_is_empty() {
        let buf = TrailBuffer::default();
        assert!(buf.points.is_empty());
    }
}
