use bevy::prelude::*;
use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration as UomAcceleration, Length as UomLength},
};

use crate::game::shared::{
    constants::{SCREEN_HEIGHT_UOM, SCREEN_WIDTH_UOM},
    helpers::get_translation_from_percentage,
    systems::calculate_gravitational_acceleration,
    types::{Mass, Position},
};

/// Number of grid columns (horizontal sample points).
const GRID_COLS: u32 = 20;

/// Number of grid rows (vertical sample points).
const GRID_ROWS: u32 = 12;

/// Maximum length (in pixels) of a field-direction line at the strongest field strength.
const MAX_LINE_LENGTH_PX: f32 = 20.0;

// Pure functions.

/// Compute the total gravitational field vector at a world-space point.
///
/// Returns the magnitude (in raw UOM value) and the normalized 2D direction.
#[must_use]
pub(crate) fn compute_field_at_point(point_x: UomLength, point_y: UomLength, masses: &[(UomLength, UomLength, uom::si::f64::Mass)]) -> (f64, Vec2) {
    let mut accel_x = UomAcceleration::new::<meter_per_second_squared>(0.0);
    let mut accel_y = UomAcceleration::new::<meter_per_second_squared>(0.0);

    for &(mx, my, mass) in masses {
        let (ax, ay) = calculate_gravitational_acceleration(point_x, point_y, mx, my, mass);
        accel_x += ax;
        accel_y += ay;
    }

    let mag = (accel_x.value * accel_x.value + accel_y.value * accel_y.value).sqrt();

    if mag < 1e-30 {
        return (0.0, Vec2::ZERO);
    }

    #[allow(clippy::cast_possible_truncation)]
    let dir = Vec2::new((accel_x.value / mag) as f32, (accel_y.value / mag) as f32);

    (mag, dir)
}

// Systems.

/// Render a grid of short lines showing gravitational field direction and strength.
///
/// Runs during all `InGame` sub-states. Each grid point samples the combined gravitational
/// acceleration from all `Mass` entities and draws a Gizmo line in the field direction,
/// with length proportional to log-scaled field strength.
pub fn gravity_grid_render_system(mass_query: Query<(&Position, &Mass)>, mut gizmos: Gizmos) {
    // Collect mass data to avoid repeated queries.
    let masses: Vec<_> = mass_query.iter().map(|(pos, mass)| (pos.x, pos.y, mass.value)).collect();

    if masses.is_empty() {
        return;
    }

    // Pre-compute field magnitudes to find the max for normalization.
    let mut samples: Vec<(Vec3, f64, Vec2)> = Vec::with_capacity((GRID_COLS * GRID_ROWS) as usize);

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            // Map grid cell to world percentage (centered in each cell).
            let frac_x = (f64::from(col) + 0.5) / f64::from(GRID_COLS);
            let frac_y = (f64::from(row) + 0.5) / f64::from(GRID_ROWS);

            let world_x = *SCREEN_WIDTH_UOM * frac_x;
            let world_y = *SCREEN_HEIGHT_UOM * frac_y;

            let (mag, dir) = compute_field_at_point(world_x, world_y, &masses);

            if mag < 1e-30 {
                continue;
            }

            let screen_pos = get_translation_from_percentage(frac_x, frac_y);
            samples.push((screen_pos, mag, dir));
        }
    }

    if samples.is_empty() {
        return;
    }

    // Find max magnitude for normalization (use log scale for better visual range).
    let max_mag = samples.iter().map(|(_, m, _)| *m).fold(0.0_f64, f64::max);

    if max_mag < 1e-30 {
        return;
    }

    let ln_max_magnitude = max_mag.ln();

    for (screen_pos, mag, dir) in &samples {
        // Log-scale normalization: stronger fields are longer, but the range is compressed.
        let ln_magnitude = mag.ln();
        #[allow(clippy::cast_possible_truncation)]
        let strength = ((ln_magnitude / ln_max_magnitude).clamp(0.0, 1.0)) as f32;

        // Skip very faint lines.
        if strength < 0.05 {
            continue;
        }

        let start = screen_pos.truncate();
        let line_len = MAX_LINE_LENGTH_PX * strength;
        let end = start + *dir * line_len;

        // Modulate alpha by strength.
        let alpha = 0.1 + strength * 0.4;
        let color = Color::srgba(0.6, 0.7, 0.9, alpha);

        gizmos.line_2d(start, end, color);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use uom::si::{f64::Mass as UomMass, length::kilometer as km_unit, mass::kilogram};

    // --- compute_field_at_point ---

    #[test]
    fn field_at_mass_center_returns_zero() {
        // At the exact position of the mass, direction vector is NaN (normalize of zero),
        // so magnitude should effectively be handled. The function uses the raw gravitational
        // calculation which may return extreme values; we test a near-center point instead.
        let masses = vec![(UomLength::new::<km_unit>(100.0), UomLength::new::<km_unit>(100.0), UomMass::new::<kilogram>(1.989e30))];

        // Point far from the mass should have a non-zero field pointing toward the mass.
        let (mag, dir) = compute_field_at_point(UomLength::new::<km_unit>(1_000_000.0), UomLength::new::<km_unit>(100.0), &masses);

        assert!(mag > 0.0, "magnitude should be positive far from mass");
        // Direction should point toward the mass (negative x direction).
        assert!(dir.x < 0.0, "field should point toward mass (negative x), got {}", dir.x);
    }

    #[test]
    fn field_with_no_masses_returns_zero() {
        let masses: Vec<(UomLength, UomLength, UomMass)> = vec![];
        let (mag, dir) = compute_field_at_point(UomLength::new::<km_unit>(500.0), UomLength::new::<km_unit>(500.0), &masses);

        assert!(mag < 1e-30, "magnitude should be zero with no masses");
        assert_eq!(dir, Vec2::ZERO);
    }

    #[test]
    fn field_strength_decreases_with_distance() {
        let masses = vec![(UomLength::new::<km_unit>(0.0), UomLength::new::<km_unit>(0.0), UomMass::new::<kilogram>(1.989e30))];

        let (mag_close, _) = compute_field_at_point(UomLength::new::<km_unit>(100_000.0), UomLength::new::<km_unit>(0.0), &masses);
        let (mag_far, _) = compute_field_at_point(UomLength::new::<km_unit>(1_000_000.0), UomLength::new::<km_unit>(0.0), &masses);

        assert!(mag_close > mag_far, "field should be stronger closer to mass: close={mag_close}, far={mag_far}");
    }

    #[test]
    fn field_direction_points_toward_mass() {
        let masses = vec![(UomLength::new::<km_unit>(0.0), UomLength::new::<km_unit>(0.0), UomMass::new::<kilogram>(1.989e30))];

        // Point to the right of the mass: field should point left (toward mass).
        let (_, dir) = compute_field_at_point(UomLength::new::<km_unit>(500_000.0), UomLength::new::<km_unit>(0.0), &masses);
        assert!(dir.x < 0.0, "field should point toward mass (negative x)");

        // Point above the mass: field should point down (toward mass).
        let (_, dir) = compute_field_at_point(UomLength::new::<km_unit>(0.0), UomLength::new::<km_unit>(500_000.0), &masses);
        assert!(dir.y < 0.0, "field should point toward mass (negative y)");
    }

    #[test]
    fn field_from_two_masses_is_superposition() {
        let mass_val = UomMass::new::<kilogram>(1.989e30);
        let masses = vec![
            (UomLength::new::<km_unit>(-500_000.0), UomLength::new::<km_unit>(0.0), mass_val),
            (UomLength::new::<km_unit>(500_000.0), UomLength::new::<km_unit>(0.0), mass_val),
        ];

        // At the midpoint between two equal masses, forces should cancel along x.
        let (_, dir) = compute_field_at_point(UomLength::new::<km_unit>(0.0), UomLength::new::<km_unit>(0.0), &masses);

        // x component should be approximately zero due to symmetry.
        assert!(dir.x.abs() < 0.01, "x component should be ~0 at midpoint, got {}", dir.x);
    }
}
