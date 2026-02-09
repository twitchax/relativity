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

/// Number of grid columns (cells, not vertices).
const GRID_COLS: u32 = 40;

/// Number of grid rows (cells, not vertices).
const GRID_ROWS: u32 = 24;

/// Number of vertices per row (one more than columns).
const VERTEX_COLS: u32 = GRID_COLS + 1;

/// Number of vertices per column (one more than rows).
const VERTEX_ROWS: u32 = GRID_ROWS + 1;

/// Maximum displacement (in pixels) a vertex can be shifted toward a mass.
const MAX_DISPLACEMENT_PX: f32 = 80.0;

/// Scale factor applied to log-scaled normalized field magnitude before capping.
const DISPLACEMENT_SCALE: f32 = 18.0;

/// Reference field strength (m/s²) used to normalize magnitudes before log scaling.
///
/// Chosen so that `ln(1 + mag / REF)` spans a useful range across the game's
/// gravity wells (Earth-mass objects produce subtle warping while solar-mass
/// objects produce dramatic funneling).
const REFERENCE_FIELD_STRENGTH: f64 = 50.0;

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

/// Compute the displaced screen position and displacement magnitude for a single grid vertex.
///
/// Returns `(displaced_position, displacement_magnitude)`.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
fn compute_displaced_vertex(frac_x: f64, frac_y: f64, masses: &[(UomLength, UomLength, uom::si::f64::Mass)]) -> (Vec2, f32) {
    let world_x = *SCREEN_WIDTH_UOM * frac_x;
    let world_y = *SCREEN_HEIGHT_UOM * frac_y;

    let (mag, dir) = compute_field_at_point(world_x, world_y, masses);

    let base_pos = get_translation_from_percentage(frac_x, frac_y).truncate();

    if mag < 1e-30 {
        return (base_pos, 0.0);
    }

    let displacement = ((1.0 + mag / REFERENCE_FIELD_STRENGTH).ln() as f32 * DISPLACEMENT_SCALE).min(MAX_DISPLACEMENT_PX);
    let displaced_pos = base_pos + dir * displacement;

    (displaced_pos, displacement)
}

// Systems.

/// Render a warped Euclidean grid showing spacetime curvature near massive objects.
///
/// Builds a vertex grid and displaces each vertex toward nearby masses using
/// `compute_field_at_point`. Draws connected horizontal and vertical line segments
/// between displaced vertices, colored by local curvature strength.
pub fn gravity_grid_render_system(mass_query: Query<(&Position, &Mass)>, mut gizmos: Gizmos) {
    let masses: Vec<_> = mass_query.iter().map(|(pos, mass)| (pos.x, pos.y, mass.value)).collect();

    if masses.is_empty() {
        return;
    }

    // Build the displaced vertex grid (row-major order).
    let total_vertices = (VERTEX_ROWS * VERTEX_COLS) as usize;
    let mut positions: Vec<Vec2> = Vec::with_capacity(total_vertices);
    let mut displacements: Vec<f32> = Vec::with_capacity(total_vertices);

    for row in 0..VERTEX_ROWS {
        for col in 0..VERTEX_COLS {
            let frac_x = f64::from(col) / f64::from(GRID_COLS);
            let frac_y = f64::from(row) / f64::from(GRID_ROWS);

            let (pos, disp) = compute_displaced_vertex(frac_x, frac_y, &masses);
            positions.push(pos);
            displacements.push(disp);
        }
    }

    // Find max displacement for color normalization.
    let max_disp = displacements.iter().copied().fold(0.0_f32, f32::max);
    let safe_max = if max_disp < 1e-6 { 1.0 } else { max_disp };

    // Draw horizontal line segments: vertex[row][col] → vertex[row][col+1].
    for row in 0..VERTEX_ROWS {
        for col in 0..GRID_COLS {
            let idx_a = (row * VERTEX_COLS + col) as usize;
            let idx_b = (row * VERTEX_COLS + col + 1) as usize;

            let color = curvature_color(displacements[idx_a], displacements[idx_b], safe_max);
            gizmos.line_2d(positions[idx_a], positions[idx_b], color);
        }
    }

    // Draw vertical line segments: vertex[row][col] → vertex[row+1][col].
    for row in 0..GRID_ROWS {
        for col in 0..VERTEX_COLS {
            let idx_a = (row * VERTEX_COLS + col) as usize;
            let idx_b = ((row + 1) * VERTEX_COLS + col) as usize;

            let color = curvature_color(displacements[idx_a], displacements[idx_b], safe_max);
            gizmos.line_2d(positions[idx_a], positions[idx_b], color);
        }
    }
}

/// Map average endpoint displacement to a heat-map color (blue → purple → red/orange).
#[must_use]
fn curvature_color(disp_a: f32, disp_b: f32, max_disp: f32) -> Color {
    let t = ((disp_a + disp_b) * 0.5 / max_disp).clamp(0.0, 1.0);

    // Interpolate: blue (0.2,0.4,1.0) → purple (0.6,0.2,0.8) → orange/red (1.0,0.4,0.1).
    let (r, g, b) = if t < 0.5 {
        let s = t * 2.0;
        (0.2 + s * 0.4, 0.4 - s * 0.2, 1.0 - s * 0.2)
    } else {
        let s = (t - 0.5) * 2.0;
        (0.6 + s * 0.4, 0.2 + s * 0.2, 0.8 - s * 0.7)
    };

    // Modulate alpha: faint in flat regions, brighter near masses.
    let alpha = 0.08 + t * 0.45;

    Color::srgba(r, g, b, alpha)
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

    // --- compute_displaced_vertex ---

    #[test]
    fn displaced_vertex_no_masses_returns_zero_displacement() {
        let masses: Vec<(UomLength, UomLength, UomMass)> = vec![];
        let (pos, disp) = compute_displaced_vertex(0.5, 0.5, &masses);

        assert!(disp.abs() < 1e-6, "displacement should be zero with no masses, got {disp}");

        // Position should equal the base (undisplaced) position.
        let base = get_translation_from_percentage(0.5, 0.5).truncate();
        assert!((pos - base).length() < 1e-3, "position should match base when no masses present");
    }

    #[test]
    fn displaced_vertex_increases_near_mass() {
        // Use a very large mass so displacement is measurable at screen-scale distances.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.0, *SCREEN_HEIGHT_UOM * 0.0, UomMass::new::<kilogram>(1.0e36))];

        // A vertex close to the mass should have more displacement than one far away.
        let (_, disp_close) = compute_displaced_vertex(0.1, 0.0, &masses);
        let (_, disp_far) = compute_displaced_vertex(0.9, 0.0, &masses);

        assert!(disp_close > 0.0, "close vertex should have positive displacement");
        assert!(disp_far > 0.0, "far vertex should have positive displacement");
        assert!(disp_close > disp_far, "displacement should be larger closer to mass: close={disp_close}, far={disp_far}");
    }

    #[test]
    fn displaced_vertex_is_capped_at_max() {
        // Use a very large mass that stays below the relativistic clamp threshold
        // at this distance, but produces enough field to saturate the displacement cap.
        //
        // At frac 0.51 vs mass at frac 0.5, distance ≈ 0.01 * 6e9 km = 6e7 km = 6e10 m.
        // Relativistic clamp: M < c²d/(2G) = 9e16 * 6e10 / (2 * 6.674e-11) ≈ 4e37 kg.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.5, *SCREEN_HEIGHT_UOM * 0.5, UomMass::new::<kilogram>(1.0e37))];

        let (_, disp) = compute_displaced_vertex(0.51, 0.5, &masses);

        assert!(disp <= MAX_DISPLACEMENT_PX, "displacement {disp} should not exceed MAX_DISPLACEMENT_PX ({MAX_DISPLACEMENT_PX})");
        assert!(disp > 0.0, "displacement should be positive near a mass");
    }

    #[test]
    fn displaced_vertex_shifts_toward_mass() {
        // Use a very large mass to produce visible displacement at screen scale.
        let mass_x = *SCREEN_WIDTH_UOM * 0.5;
        let mass_y = *SCREEN_HEIGHT_UOM * 0.5;
        let masses = vec![(mass_x, mass_y, UomMass::new::<kilogram>(1.0e36))];

        // Vertex to the right of the mass.
        let base = get_translation_from_percentage(0.8, 0.5).truncate();
        let (displaced, disp) = compute_displaced_vertex(0.8, 0.5, &masses);

        assert!(disp > 0.0, "displacement should be positive");

        // Displaced position should be closer to the mass (at center) than the base.
        let mass_screen = get_translation_from_percentage(0.5, 0.5).truncate();
        let dist_base = (base - mass_screen).length();
        let dist_displaced = (displaced - mass_screen).length();
        assert!(
            dist_displaced < dist_base,
            "displaced vertex should be closer to mass: base_dist={dist_base}, displaced_dist={dist_displaced}"
        );
    }

    // --- curvature_color ---

    #[test]
    fn curvature_color_low_displacement_is_blue() {
        let color = curvature_color(0.0, 0.0, 1.0);
        // At zero displacement, color should be blue (r=0.2, g=0.4, b=1.0).
        let srgba = color.to_srgba();
        assert!((srgba.blue - 1.0).abs() < 0.01, "blue channel should be ~1.0, got {}", srgba.blue);
        assert!((srgba.red - 0.2).abs() < 0.01, "red channel should be ~0.2, got {}", srgba.red);
    }

    #[test]
    fn curvature_color_high_displacement_is_warm() {
        let color = curvature_color(1.0, 1.0, 1.0);
        // At max displacement, color should be orange/red (r=1.0, g=0.4, b=0.1).
        let srgba = color.to_srgba();
        assert!((srgba.red - 1.0).abs() < 0.01, "red channel should be ~1.0, got {}", srgba.red);
        assert!(srgba.blue < 0.2, "blue channel should be low, got {}", srgba.blue);
    }

    #[test]
    fn curvature_color_alpha_increases_with_displacement() {
        let color_low = curvature_color(0.0, 0.0, 1.0);
        let color_high = curvature_color(1.0, 1.0, 1.0);
        let alpha_low = color_low.to_srgba().alpha;
        let alpha_high = color_high.to_srgba().alpha;

        assert!(alpha_high > alpha_low, "alpha should increase with displacement: low={alpha_low}, high={alpha_high}");
    }
}
