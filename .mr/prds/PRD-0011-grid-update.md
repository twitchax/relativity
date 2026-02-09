---
id: PRD-0011
title: "Warped Euclidean Gravity Grid"
status: draft
owner: twitchax
created: 2026-02-09
updated: 2026-02-09
principles:
  - "Visual-only change: do not alter game physics or the compute_field_at_point logic"
  - "Reuse existing gravitational field computation for displacement calculations"
  - "Displacement-based warping with smooth falloff for organic curvature appearance"
  - "Grid density 40x24 for smooth visual fidelity"
  - "Curvature heat-map coloring: blue (flat) → purple → red/orange (strong curvature)"
  - "Dramatic but reasonable funneling proportional to gravity well strength"
references:
  - name: "Current gravity grid implementation"
    url: "src/game/gravity_grid/mod.rs"
  - name: "Gravitational acceleration helper"
    url: "src/game/shared/systems.rs"
  - name: "Screen coordinate helpers"
    url: "src/game/shared/helpers.rs"
  - name: "Game constants (screen dimensions)"
    url: "src/game/shared/constants.rs"
acceptance_tests:
  - id: uat-001
    name: "Grid renders as connected horizontal and vertical lines (not disconnected arrows)"
    command: cargo make uat
    uat_status: unverified
  - id: uat-002
    name: "Grid vertices are displaced toward massive objects proportional to field strength"
    command: cargo make uat
    uat_status: unverified
  - id: uat-003
    name: "Grid lines change color based on local curvature (blue → purple → red/orange)"
    command: cargo make uat
    uat_status: unverified
  - id: uat-004
    name: "Grid is uniform and evenly spaced in regions with no gravitational influence"
    command: cargo make uat
    uat_status: unverified
  - id: uat-005
    name: "Larger gravity wells produce more dramatic funneling than smaller ones"
    command: cargo make uat
    uat_status: unverified
  - id: uat-006
    name: "Existing unit tests for compute_field_at_point continue to pass"
    command: cargo make uat
    uat_status: unverified
tasks:
  - id: T-001
    title: "Replace vector-field rendering with warped Euclidean grid vertex computation"
    priority: 1
    status: todo
    notes: "Build a 41x25 vertex grid (40x24 cells). For each vertex, compute uniform screen position, then apply displacement toward masses using compute_field_at_point. Use log-scaled displacement with a tunable max displacement cap."
  - id: T-002
    title: "Render grid as connected horizontal and vertical line segments between displaced vertices"
    priority: 1
    status: todo
    notes: "Use gizmos.line_2d to draw lines between adjacent displaced vertices — horizontal lines along rows, vertical lines along columns."
  - id: T-003
    title: "Implement curvature-based heat-map coloring for grid lines"
    priority: 2
    status: todo
    notes: "Color each line segment based on the average displacement magnitude of its two endpoints. Map from blue (flat space, low displacement) through purple to red/orange (strong curvature, high displacement)."
  - id: T-004
    title: "Tune displacement magnitude and falloff for visible but reasonable funneling"
    priority: 2
    status: todo
    notes: "Use log-scaled displacement with a smooth inverse-square falloff. Add a MAX_DISPLACEMENT_PX constant to cap warping. Ensure stronger gravity wells produce proportionally more funneling."
  - id: T-005
    title: "Update or add unit tests for grid vertex displacement logic"
    priority: 3
    status: todo
    notes: "Test that displacement is zero when no masses are present, displacement increases near masses, and displacement is capped at the maximum."
---

# Summary

Replace the current gravity grid visualization (a vector field of short directional lines) with a warped Euclidean grid — a regular rectangular mesh that visually deforms near massive objects. Grid lines converge and funnel toward gravity wells, with color shifting from blue (flat space) to red/orange (strong curvature) via a heat-map gradient.

# Problem

The current gravity grid renders as disconnected directional arrows (a vector field), which doesn't intuitively convey spacetime curvature. A warped grid is the canonical visualization for gravitational effects in general relativity and communicates the concept of curved spacetime much more effectively to players.

# Goals

1. Render a 40×24 Euclidean grid that warps visibly near massive objects using displacement-based vertex shifting.
2. Apply curvature heat-map coloring (blue → purple → red/orange) to grid lines based on local gravitational field strength.
3. Ensure funneling is proportional to gravity well strength — larger masses produce more dramatic warping.
4. Maintain uniform, evenly-spaced grid appearance in regions with negligible gravitational influence.
5. Reuse the existing `compute_field_at_point` function for displacement calculations.

# Technical Approach

The implementation replaces the body of `gravity_grid_render_system` in `src/game/gravity_grid/mod.rs` while keeping `compute_field_at_point` unchanged.

## Vertex Grid Layout

Construct a (GRID_COLS+1) × (GRID_ROWS+1) = 41×25 vertex grid. Each vertex starts at a uniform screen position computed from fractional coordinates, identical to the current approach.

## Displacement

For each vertex, call `compute_field_at_point` to get the gravitational acceleration vector. Displace the vertex **in the direction of the field** (toward the mass) by an amount proportional to the log-scaled magnitude:

```
displacement = direction * min(log(1 + magnitude) * SCALE_FACTOR, MAX_DISPLACEMENT_PX)
```

Log scaling compresses the dynamic range so that near-field vertices don't fly off-screen while far-field vertices still show visible warping.

## Rendering

Draw connected line segments between adjacent displaced vertices:
- Horizontal lines: vertex[row][col] → vertex[row][col+1]
- Vertical lines: vertex[row][col] → vertex[row+1][col]

## Coloring

Each line segment is colored based on the average displacement magnitude of its two endpoints, mapped through a heat-map gradient:
- **Low displacement** (flat space): blue `(0.2, 0.4, 1.0)`
- **Medium displacement**: purple `(0.6, 0.2, 0.8)`
- **High displacement** (strong curvature): orange/red `(1.0, 0.4, 0.1)`

Alpha is also modulated by displacement strength (faint in flat regions, brighter near masses).

```
┌──────────────────────────────────────────────┐
│  Uniform grid (no masses)                    │
│  ┼───┼───┼───┼───┼───┼───┼───┼───┼          │
│  │   │   │   │   │   │   │   │   │          │
│  ┼───┼───┼───┼───┼───┼───┼───┼───┼          │
│  │   │   │   │   │   │   │   │   │          │
│  ┼───┼───┼───┼───┼───┼───┼───┼───┼          │
│                                              │
│  Warped grid (mass at center)                │
│  ┼───┼───┼──┼─┼┼─┼──┼───┼───┼               │
│  │   │   │  │╲│╱│  │   │   │               │
│  ┼───┼───┼─┼╲ ● ╱┼─┼───┼───┼               │
│  │   │   │  │╱│╲│  │   │   │               │
│  ┼───┼───┼──┼─┼┼─┼──┼───┼───┼               │
└──────────────────────────────────────────────┘
```

# Assumptions

- The existing `compute_field_at_point` function provides sufficient field data for displacement calculations.
- A 40×24 grid (41×25 = 1,025 vertices) is performant enough for real-time gizmo rendering.
- Log-scaled displacement with a tunable cap will produce the desired "visible but reasonable" funneling aesthetic.

# Constraints

- Must not modify `compute_field_at_point` or any game physics logic.
- Must continue to work with all existing levels and mass configurations.
- Grid rendering must remain in the gizmo system (no spawned entities or meshes).
- Existing unit tests for `compute_field_at_point` must continue to pass.

# References to Code

- `src/game/gravity_grid/mod.rs` — Current gravity grid rendering (to be replaced).
- `src/game/shared/systems.rs` — `calculate_gravitational_acceleration` used by `compute_field_at_point`.
- `src/game/shared/helpers.rs` — `get_translation_from_percentage` for coordinate mapping.
- `src/game/shared/constants.rs` — `SCREEN_WIDTH_UOM`, `SCREEN_HEIGHT_UOM` for screen dimensions.
- `src/game/mod.rs` — System registration in `GamePlugin`.

# Non-Goals (MVP)

- 3D perspective or depth-based warping (this is a 2D top-down visualization).
- Animated grid transitions when masses move.
- Geodesic/metric-based line curvature (displacement-based is sufficient).
- Configurable grid density at runtime.
- Subdivision or Bézier smoothing of line segments between vertices.

# History
