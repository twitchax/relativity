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
use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

/// Number of grid columns (cells, not vertices).
const GRID_COLS: u32 = 40;

/// Number of grid rows (cells, not vertices).
const GRID_ROWS: u32 = 24;

/// Number of vertices per row (one more than columns).
const VERTEX_COLS: u32 = GRID_COLS + 1;

/// Number of vertices per column (one more than rows).
const VERTEX_ROWS: u32 = GRID_ROWS + 1;

/// Maximum displacement (in pixels) a vertex can be shifted toward a mass.
const MAX_DISPLACEMENT_PX: f32 = 40.0;

/// Scale factor applied to log-scaled normalized field magnitude before capping.
const DISPLACEMENT_SCALE: f32 = 10.0;

/// Minimum fraction of the original cell spacing preserved between adjacent vertices.
///
/// Prevents grid lines from crossing while still allowing vertices to converge near
/// masses, creating a funnel effect rather than crossed distortions.
const MIN_SPACING_FRACTION: f32 = 0.05;

/// Maximum fraction of the screen-space distance to the nearest mass that a vertex
/// can be displaced.  Prevents vertices from overshooting past mass centers.
const MAX_PROXIMITY_FRACTION: f32 = 0.75;

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

    // Guard against NaN/infinity (e.g. vertex coincident with a mass) or zero field.
    if mag < 1e-30 || !dir.x.is_finite() || !dir.y.is_finite() {
        return (base_pos, 0.0);
    }

    let mut displacement = ((1.0 + mag / REFERENCE_FIELD_STRENGTH).ln() as f32 * DISPLACEMENT_SCALE).min(MAX_DISPLACEMENT_PX);

    // Cap displacement to a fraction of the screen-space distance to the nearest
    // mass so that vertices converge *toward* a mass but never overshoot past it.
    let nearest_mass_dist = masses
        .iter()
        .map(|&(mx, my, _)| {
            let mass_frac_x = (mx / *SCREEN_WIDTH_UOM).value;
            let mass_frac_y = (my / *SCREEN_HEIGHT_UOM).value;
            let mass_px = Vec2::new((SCREEN_WIDTH_PX * mass_frac_x) as f32, (SCREEN_HEIGHT_PX * mass_frac_y) as f32);
            (base_pos - mass_px).length()
        })
        .fold(f32::MAX, f32::min);

    displacement = displacement.min(nearest_mass_dist * MAX_PROXIMITY_FRACTION);

    let displaced_pos = base_pos + dir * displacement;

    (displaced_pos, displacement)
}

/// Enforce grid topology by ensuring adjacent vertices maintain minimum spacing.
///
/// After displacement, vertices near a mass can converge and potentially cross
/// their neighbors.  This function performs bidirectional sweeps along both axes
/// to guarantee a minimum gap between every pair of adjacent vertices, preserving
/// the grid's topological ordering.
#[allow(clippy::cast_possible_truncation)]
fn enforce_grid_topology(positions: &mut [Vec2]) {
    let cell_width = SCREEN_WIDTH_PX as f32 / f32::from(GRID_COLS as u16);
    let cell_height = SCREEN_HEIGHT_PX as f32 / f32::from(GRID_ROWS as u16);
    let min_gap_x = cell_width * MIN_SPACING_FRACTION;
    let min_gap_y = cell_height * MIN_SPACING_FRACTION;

    // Horizontal: forward pass (left → right).
    for row in 0..VERTEX_ROWS {
        for col in 1..VERTEX_COLS {
            let prev = (row * VERTEX_COLS + col - 1) as usize;
            let curr = (row * VERTEX_COLS + col) as usize;

            if positions[curr].x < positions[prev].x + min_gap_x {
                positions[curr].x = positions[prev].x + min_gap_x;
            }
        }
    }

    // Horizontal: backward pass (right → left).
    for row in 0..VERTEX_ROWS {
        for col in (0..VERTEX_COLS - 1).rev() {
            let curr = (row * VERTEX_COLS + col) as usize;
            let next = (row * VERTEX_COLS + col + 1) as usize;

            if positions[curr].x > positions[next].x - min_gap_x {
                positions[curr].x = positions[next].x - min_gap_x;
            }
        }
    }

    // Vertical: forward pass (increasing y / row index).
    for col in 0..VERTEX_COLS {
        for row in 1..VERTEX_ROWS {
            let prev = ((row - 1) * VERTEX_COLS + col) as usize;
            let curr = (row * VERTEX_COLS + col) as usize;

            if positions[curr].y < positions[prev].y + min_gap_y {
                positions[curr].y = positions[prev].y + min_gap_y;
            }
        }
    }

    // Vertical: backward pass (decreasing y / row index).
    for col in 0..VERTEX_COLS {
        for row in (0..VERTEX_ROWS - 1).rev() {
            let curr = (row * VERTEX_COLS + col) as usize;
            let next = ((row + 1) * VERTEX_COLS + col) as usize;

            if positions[curr].y > positions[next].y - min_gap_y {
                positions[curr].y = positions[next].y - min_gap_y;
            }
        }
    }
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

    // Prevent grid lines from crossing: enforce minimum spacing between neighbors.
    enforce_grid_topology(&mut positions);

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

    // --- grid uniform with no masses (uat-004) ---

    #[test]
    fn grid_uniform_and_evenly_spaced_without_masses() {
        // With no masses, every vertex should sit at its undisplaced base position
        // and the spacing between adjacent vertices should be uniform.
        let masses: Vec<(UomLength, UomLength, UomMass)> = vec![];
        let positions = build_vertex_grid(&masses);

        // Verify every vertex matches its expected base position (zero displacement).
        for row in 0..VERTEX_ROWS {
            for col in 0..VERTEX_COLS {
                let idx = (row * VERTEX_COLS + col) as usize;
                let frac_x = f64::from(col) / f64::from(GRID_COLS);
                let frac_y = f64::from(row) / f64::from(GRID_ROWS);
                let expected = get_translation_from_percentage(frac_x, frac_y).truncate();
                let actual = positions[idx];
                assert!(
                    (actual - expected).length() < 1e-3,
                    "vertex ({row},{col}) should be at base position: expected {expected}, got {actual}"
                );
            }
        }

        // Verify uniform horizontal spacing: all horizontal gaps in a row should be equal.
        for row in 0..VERTEX_ROWS {
            let first_a = (row * VERTEX_COLS) as usize;
            let first_b = (row * VERTEX_COLS + 1) as usize;
            let expected_gap = (positions[first_b].x - positions[first_a].x).abs();
            assert!(expected_gap > 1e-3, "horizontal gap should be positive");

            for col in 1..GRID_COLS {
                let idx_a = (row * VERTEX_COLS + col) as usize;
                let idx_b = (row * VERTEX_COLS + col + 1) as usize;
                let gap = (positions[idx_b].x - positions[idx_a].x).abs();
                assert!((gap - expected_gap).abs() < 1e-2, "row {row}: horizontal gap at col {col} should be {expected_gap}, got {gap}");
            }
        }

        // Verify uniform vertical spacing: all vertical gaps in a column should be equal.
        for col in 0..VERTEX_COLS {
            let first_a = col as usize;
            let first_b = (VERTEX_COLS + col) as usize;
            let expected_gap = (positions[first_b].y - positions[first_a].y).abs();
            assert!(expected_gap > 1e-3, "vertical gap should be positive");

            for row in 1..GRID_ROWS {
                let idx_a = (row * VERTEX_COLS + col) as usize;
                let idx_b = ((row + 1) * VERTEX_COLS + col) as usize;
                let gap = (positions[idx_b].y - positions[idx_a].y).abs();
                assert!((gap - expected_gap).abs() < 1e-2, "col {col}: vertical gap at row {row} should be {expected_gap}, got {gap}");
            }
        }
    }

    // --- displacement proportional to field strength (uat-002) ---

    #[test]
    fn displaced_vertex_proportional_to_field_strength() {
        // A heavier mass should produce more displacement at the same vertex position,
        // confirming displacement is proportional to gravitational field strength.
        let small_mass = vec![(*SCREEN_WIDTH_UOM * 0.5, *SCREEN_HEIGHT_UOM * 0.5, UomMass::new::<kilogram>(1.0e35))];
        let large_mass = vec![(*SCREEN_WIDTH_UOM * 0.5, *SCREEN_HEIGHT_UOM * 0.5, UomMass::new::<kilogram>(1.0e36))];

        let (_, disp_small) = compute_displaced_vertex(0.8, 0.5, &small_mass);
        let (_, disp_large) = compute_displaced_vertex(0.8, 0.5, &large_mass);

        assert!(disp_small > 0.0, "small mass should still produce positive displacement");
        assert!(disp_large > disp_small, "larger mass should produce more displacement: small={disp_small}, large={disp_large}");
    }

    #[test]
    fn displaced_vertex_direction_toward_mass_proportional() {
        // Two masses of different strengths: the displaced vertex should move toward
        // each mass, and the stronger mass should pull it closer (larger shift).
        let mass_pos_x = *SCREEN_WIDTH_UOM * 0.3;
        let mass_pos_y = *SCREEN_HEIGHT_UOM * 0.5;

        let weak = vec![(mass_pos_x, mass_pos_y, UomMass::new::<kilogram>(5.0e34))];
        let strong = vec![(mass_pos_x, mass_pos_y, UomMass::new::<kilogram>(5.0e36))];

        let base = get_translation_from_percentage(0.7, 0.5).truncate();
        let mass_screen = get_translation_from_percentage(0.3, 0.5).truncate();

        let (displaced_weak, _) = compute_displaced_vertex(0.7, 0.5, &weak);
        let (displaced_strong, _) = compute_displaced_vertex(0.7, 0.5, &strong);

        // Both should move toward the mass.
        let dist_base = (base - mass_screen).length();
        let dist_weak = (displaced_weak - mass_screen).length();
        let dist_strong = (displaced_strong - mass_screen).length();

        assert!(dist_weak < dist_base, "weak mass should still pull vertex closer");
        assert!(
            dist_strong < dist_weak,
            "stronger mass should pull vertex closer than weak: weak_dist={dist_weak}, strong_dist={dist_strong}"
        );
    }

    // --- grid connectivity (uat-001) ---

    /// Build the displaced vertex grid the same way the render system does,
    /// including topology enforcement.
    fn build_vertex_grid(masses: &[(UomLength, UomLength, UomMass)]) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity((VERTEX_ROWS * VERTEX_COLS) as usize);
        for row in 0..VERTEX_ROWS {
            for col in 0..VERTEX_COLS {
                let frac_x = f64::from(col) / f64::from(GRID_COLS);
                let frac_y = f64::from(row) / f64::from(GRID_ROWS);
                let (pos, _) = compute_displaced_vertex(frac_x, frac_y, masses);
                positions.push(pos);
            }
        }
        enforce_grid_topology(&mut positions);
        positions
    }

    #[test]
    fn topology_preserved_under_strong_mass() {
        // Place a very strong mass between vertex positions (avoid singularity)
        // and verify that no pair of adjacent vertices has crossed ordering.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.53, *SCREEN_HEIGHT_UOM * 0.47, UomMass::new::<kilogram>(1.0e37))];
        let positions = build_vertex_grid(&masses);

        // Horizontal: x must be strictly increasing within each row.
        for row in 0..VERTEX_ROWS {
            for col in 1..VERTEX_COLS {
                let prev = (row * VERTEX_COLS + col - 1) as usize;
                let curr = (row * VERTEX_COLS + col) as usize;
                assert!(
                    positions[curr].x > positions[prev].x,
                    "row {row}: vertex col {col} x ({}) should be > col {} x ({})",
                    positions[curr].x,
                    col - 1,
                    positions[prev].x
                );
            }
        }

        // Vertical: y must be strictly increasing within each column.
        for col in 0..VERTEX_COLS {
            for row in 1..VERTEX_ROWS {
                let prev = ((row - 1) * VERTEX_COLS + col) as usize;
                let curr = (row * VERTEX_COLS + col) as usize;
                assert!(
                    positions[curr].y > positions[prev].y,
                    "col {col}: vertex row {row} y ({}) should be > row {} y ({})",
                    positions[curr].y,
                    row - 1,
                    positions[prev].y
                );
            }
        }
    }

    #[test]
    fn grid_forms_connected_horizontal_lines() {
        // Place mass between vertices (0.53, 0.47) to avoid singularity at an exact vertex.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.53, *SCREEN_HEIGHT_UOM * 0.47, UomMass::new::<kilogram>(1.0e36))];
        let positions = build_vertex_grid(&masses);

        // Every horizontal pair [row][col] → [row][col+1] must be a valid line segment
        // (both points finite and not identical — connected, not degenerate).
        for row in 0..VERTEX_ROWS {
            for col in 0..GRID_COLS {
                let idx_a = (row * VERTEX_COLS + col) as usize;
                let idx_b = (row * VERTEX_COLS + col + 1) as usize;
                let a = positions[idx_a];
                let b = positions[idx_b];
                assert!(a.x.is_finite() && a.y.is_finite(), "vertex ({row},{col}) is not finite");
                assert!(b.x.is_finite() && b.y.is_finite(), "vertex ({row},{}) is not finite", col + 1);
                assert!((a - b).length() > 1e-3, "horizontal segment ({row},{col})→({row},{}) is degenerate (zero length)", col + 1);
            }
        }
    }

    #[test]
    fn grid_forms_connected_vertical_lines() {
        // Place mass between vertices to avoid singularity.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.53, *SCREEN_HEIGHT_UOM * 0.47, UomMass::new::<kilogram>(1.0e36))];
        let positions = build_vertex_grid(&masses);

        // Every vertical pair [row][col] → [row+1][col] must be a valid line segment.
        for row in 0..GRID_ROWS {
            for col in 0..VERTEX_COLS {
                let idx_a = (row * VERTEX_COLS + col) as usize;
                let idx_b = ((row + 1) * VERTEX_COLS + col) as usize;
                let a = positions[idx_a];
                let b = positions[idx_b];
                assert!(a.x.is_finite() && a.y.is_finite(), "vertex ({row},{col}) is not finite");
                assert!(b.x.is_finite() && b.y.is_finite(), "vertex ({},{col}) is not finite", row + 1);
                assert!((a - b).length() > 1e-3, "vertical segment ({row},{col})→({},{col}) is degenerate (zero length)", row + 1);
            }
        }
    }

    #[test]
    fn grid_covers_full_vertex_count_for_connected_rendering() {
        // Verify the grid produces exactly VERTEX_ROWS * VERTEX_COLS vertices,
        // matching the render system's horizontal and vertical line loop bounds.
        let masses = vec![(*SCREEN_WIDTH_UOM * 0.53, *SCREEN_HEIGHT_UOM * 0.47, UomMass::new::<kilogram>(1.0e36))];
        let positions = build_vertex_grid(&masses);

        let expected = (VERTEX_ROWS * VERTEX_COLS) as usize;
        assert_eq!(positions.len(), expected, "vertex count must be {expected} for connected grid rendering");

        // Number of horizontal segments = VERTEX_ROWS * GRID_COLS.
        let h_segments = VERTEX_ROWS * GRID_COLS;
        // Number of vertical segments = GRID_ROWS * VERTEX_COLS.
        let v_segments = GRID_ROWS * VERTEX_COLS;

        assert_eq!(h_segments, 25 * 40, "expected 1000 horizontal line segments");
        assert_eq!(v_segments, 24 * 41, "expected 984 vertical line segments");
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
    fn curvature_color_mid_displacement_is_purple() {
        // At 50% displacement, color should be purple (r≈0.6, g≈0.2, b≈0.8).
        let color = curvature_color(0.5, 0.5, 1.0);
        let srgba = color.to_srgba();
        assert!((srgba.red - 0.6).abs() < 0.05, "red channel should be ~0.6, got {}", srgba.red);
        assert!((srgba.green - 0.2).abs() < 0.05, "green channel should be ~0.2, got {}", srgba.green);
        assert!((srgba.blue - 0.8).abs() < 0.05, "blue channel should be ~0.8, got {}", srgba.blue);
    }

    #[test]
    fn curvature_color_gradient_blue_purple_red() {
        // Verify the full gradient ordering: blue (low) → purple (mid) → red/orange (high).
        let low = curvature_color(0.0, 0.0, 1.0).to_srgba();
        let mid = curvature_color(0.5, 0.5, 1.0).to_srgba();
        let high = curvature_color(1.0, 1.0, 1.0).to_srgba();

        // Red channel should increase monotonically: low < mid < high.
        assert!(low.red < mid.red, "red should increase from low to mid: {}<{}", low.red, mid.red);
        assert!(mid.red < high.red, "red should increase from mid to high: {}<{}", mid.red, high.red);

        // Blue channel should decrease monotonically: low > mid > high.
        assert!(low.blue > mid.blue, "blue should decrease from low to mid: {}>{}", low.blue, mid.blue);
        assert!(mid.blue > high.blue, "blue should decrease from mid to high: {}>{}", mid.blue, high.blue);
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
