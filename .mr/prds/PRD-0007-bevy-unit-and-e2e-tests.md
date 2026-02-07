---
id: PRD-0007
title: "Bevy Unit and E2E Tests"
status: active
owner: twitchax
created: 2026-02-07
updated: 2026-02-07
principles:
  - "Extract pure logic from Bevy systems to enable direct unit testing"
  - "Use MinimalPlugins for headless E2E tests — add only what each test needs"
  - "Prioritize physics and relativity math for highest-value test coverage"
  - "Refactoring must not change observable game behavior"
  - "Tests must run in CI without a GPU or display server"
references:
  - name: "Bevy MinimalPlugins docs"
    url: "https://docs.rs/bevy/latest/bevy/struct.MinimalPlugins.html"
  - name: "Bevy headless testing guide"
    url: "https://taintedcoders.com/bevy/how-to/headless-mode"
  - name: "Bevy headless rendering (offscreen render target)"
    url: "https://github.com/bevyengine/bevy/issues/3155"
  - name: "approx crate"
    url: "https://docs.rs/approx/latest/approx/"
  - name: "proptest crate"
    url: "https://docs.rs/proptest/latest/proptest/"
acceptance_tests:
  - id: uat-001
    name: "All unit tests pass"
    command: cargo make test
    uat_status: unverified
  - id: uat-002
    name: "All E2E headless tests pass"
    command: cargo make test
    uat_status: unverified
  - id: uat-003
    name: "CI pipeline passes (fmt-check, clippy, test)"
    command: cargo make ci
    uat_status: unverified
  - id: uat-004
    name: "Headless render smoke test passes without a window or display"
    command: cargo make test
    uat_status: unverified
tasks:
  - id: T-001
    title: "Add dev-dependencies: approx and proptest"
    priority: 1
    status: done
    notes: "Add approx (float comparison) and proptest (property-based testing) to Cargo.toml [dev-dependencies]"
  - id: T-002
    title: "Unit tests for game/shared/helpers.rs (7 pure functions)"
    priority: 1
    status: done
    notes: "Test has_collided, get_translation_from_position, get_translation_from_percentage, get_position_from_percentage, length_to_pixel, planet_sprite_pixel_radius_to_scale, rocket_sprite_pixel_radius_to_scale with normal, edge, and boundary cases"
  - id: T-003
    title: "Unit tests for game/shared/types.rs (Velocity::scalar)"
    priority: 1
    status: done
    notes: "Test Velocity::scalar() with zero, unit, and Pythagorean triple inputs"
  - id: T-004
    title: "Unit tests for game/shared/constants.rs (value validation)"
    priority: 2
    status: done
    notes: "Validate physical constants are reasonable: mass_of_sun > mass_of_earth, max_velocity < C, G and C are positive, LazyLock statics initialize correctly"
  - id: T-005
    title: "Extract and test relativistic gamma calculations from player_clock.rs"
    priority: 1
    status: done
    notes: "Extract calculate_velocity_gamma, calculate_gravitational_gamma, and calculate_player_clock as pure functions. Test: gamma >= 1 always, gamma == 1 at rest, gamma increases with velocity, gravitational gamma near massive bodies"
  - id: T-006
    title: "Extract and test gravitational acceleration from game/shared/systems.rs"
    priority: 1
    status: done
    notes: "Extract gravitational acceleration calculation and relativistic velocity adjustment from velocity_update() into pure functions. Test: force proportional to mass, inverse-square distance, direction toward mass"
  - id: T-007
    title: "Extract and test rocket rotation calculation from game/shared/systems.rs"
    priority: 2
    status: done
    notes: "Extract rotation angle calculation from rocket_rotation_update() into a pure function. Test: angle correct for cardinal and diagonal velocities"
  - id: T-008
    title: "Extract and test launch velocity calculation from player_sprite.rs"
    priority: 2
    status: done
    notes: "Extract launch velocity/direction calculation from player_launch() into a pure function. Test: respects max velocity, direction toward cursor, power scales with distance"
  - id: T-009
    title: "Extract and test observer clock formatting from observer/mod.rs"
    priority: 3
    status: todo
    notes: "Extract time formatting into a pure function. Test: correct format string output"
  - id: T-010
    title: "Add proptest property-based tests for physics invariants"
    priority: 2
    status: done
    notes: "Property tests: velocity gamma >= 1 for all velocities < C, gravitational gamma >= 1 for all positive masses/distances, has_collided is symmetric, scalar() >= 0 for all inputs"
  - id: T-011
    title: "Create test helper module for shared Bevy test setup"
    priority: 2
    status: done
    notes: "Create src/game/test_helpers.rs (or similar) with helper functions to build a MinimalPlugins test app, spawn test entities with Position/Velocity/Mass/Radius, and run N frames"
  - id: T-012
    title: "E2E headless test: level spawning creates expected entities"
    priority: 2
    status: todo
    notes: "Use MinimalPlugins + TransformPlugin. Spawn level 1 and TimeWarp, then query world to verify correct number of entities with expected components (Player, Planet, Destination, Observer)"
  - id: T-013
    title: "E2E headless test: collision triggers level completion"
    priority: 3
    status: todo
    notes: "Spawn player and destination at overlapping positions, run collision_check system, verify GameState transitions to Finished"
  - id: T-014
    title: "E2E headless test: velocity_update applies gravity correctly"
    priority: 3
    status: todo
    notes: "Spawn player and planet, run velocity_update for several frames, verify player velocity increases toward planet"
  - id: T-015
    title: "E2E headless test: player clock experiences time dilation"
    priority: 3
    status: todo
    notes: "Spawn player with nonzero velocity, run player_clock_update, verify player clock runs slower than observer clock (time dilation effect)"
  - id: T-016
    title: "Unit tests for shared/state.rs enums (AppState, GameState)"
    priority: 3
    status: todo
    notes: "Verify enum variants exist, default values are correct, and Debug/Clone derive works"
  - id: T-017
    title: "E2E headless test: despawn_level cleans up all GameItem entities"
    priority: 3
    status: todo
    notes: "Spawn a level, run despawn_level, verify no entities with GameItem component remain"
  - id: T-018
    title: "Headless render smoke test: DefaultPlugins without a window"
    priority: 1
    status: done
    notes: "Add at least one integration test that builds a Bevy App with DefaultPlugins but disables window creation (WindowPlugin { primary_window: None } + disable WinitPlugin) and uses ScheduleRunnerPlugin. This exercises the full render pipeline (asset loading, transforms, cameras) headlessly. The test should spawn a camera rendering to an offscreen Image target, run several app.update() cycles, and assert no panics. This replaces the old manual 'cargo run' UAT with an automated, CI-safe smoke test that proves the game bootstraps and renders without a display server or GPU window."
---

# Summary

Add comprehensive unit tests and headless E2E tests to the relativity game. This involves extracting pure logic from Bevy systems into standalone testable functions, adding unit tests for all pure game logic (physics, collision, relativity math), creating headless integration tests using Bevy's `MinimalPlugins` to verify end-to-end game behavior without a GPU or display, and adding at least one headless render smoke test using `DefaultPlugins` (with window creation disabled) to verify the game bootstraps and the render graph executes — replacing the unsustainable manual `cargo run` UAT.

# Problem

The project currently has only 2 trivial unit tests covering enum existence. All core game logic — relativistic time dilation, gravitational physics, collision detection, player movement, and level spawning — is untested. This makes it risky to refactor or add features, and there is no safety net for regressions. The physics calculations are especially critical since the entire game premise depends on correct relativity math.

# Goals

1. Extract pure computation logic from Bevy systems into standalone functions that can be unit tested directly.
2. Achieve 60–80% test coverage across the codebase through unit tests on pure functions.
3. Add property-based tests (via `proptest`) for physics invariants (e.g., gamma ≥ 1, collision symmetry).
4. Create a reusable test helper module for headless Bevy app setup using `MinimalPlugins`.
5. Add E2E headless tests that verify game behavior: level spawning, collision, gravity, time dilation, and cleanup.
6. Add at least one headless render smoke test using `DefaultPlugins` (with window/winit disabled) to replace the manual `cargo run` UAT with an automated, CI-safe check.
7. All tests must run in CI without a GPU or display server.
8. Refactoring must not change any observable game behavior.

# Technical Approach

## Phase 1: Foundation (T-001, T-011, T-018)

Add `approx` and `proptest` as dev-dependencies. Create a test helper module with utilities for building headless Bevy test apps. Add the headless render smoke test early as it validates the overall approach and replaces the manual `cargo run` gate.

```
[dev-dependencies]
approx = "0.5"
proptest = "1"
```

The test helper module provides:
- A function to create a `MinimalPlugins` app with `TimePlugin` and `TransformPlugin`.
- Helper functions to spawn entities with game components (`Position`, `Velocity`, `Mass`, `Radius`).
- A function to advance the app by N update cycles.

## Phase 2: Pure Function Extraction and Unit Tests (T-002 through T-009)

Extract embedded calculations from Bevy systems into pure `pub(crate)` functions, then unit test them. The Bevy systems become thin wrappers that call the pure functions.

```
┌─────────────────────────────────────────────────────┐
│                  Bevy System                        │
│  (queries ECS, calls pure fn, writes back to ECS)  │
└──────────────────────┬──────────────────────────────┘
                       │ calls
                       ▼
┌─────────────────────────────────────────────────────┐
│               Pure Function                         │
│  (takes values, returns values, no ECS dependency)  │
│  ← unit tested directly                            │
└─────────────────────────────────────────────────────┘
```

Key extractions:

| Source System | Extracted Pure Function(s) | Module |
|---|---|---|
| `player_clock_update` | `calculate_velocity_gamma(vx, vy, c)` | player/player_clock.rs |
| `player_clock_update` | `calculate_gravitational_gamma(pos, masses, g, c)` | player/player_clock.rs |
| `player_clock_update` | `calculate_player_clock(dt, v_gamma, g_gamma, prev)` | player/player_clock.rs |
| `velocity_update` | `calculate_gravitational_acceleration(pos, other_pos, mass)` | game/shared/systems.rs |
| `rocket_rotation_update` | `calculate_rocket_rotation(vx, vy)` | game/shared/systems.rs |
| `player_launch` | `calculate_launch_velocity(cursor, player, width, max_v)` | player/player_sprite.rs |
| `observer_clock_text_update` | `format_observer_time(days)` | observer/mod.rs |

## Phase 3: Property-Based Tests (T-010)

Use `proptest` to verify physics invariants hold for arbitrary inputs:
- `velocity_gamma >= 1.0` for all `|v| < c`
- `gravitational_gamma >= 1.0` for all positive masses and distances
- `has_collided(a, b) == has_collided(b, a)` (symmetry)
- `Velocity::scalar() >= 0.0` for all inputs

## Phase 4: E2E Headless Tests (T-012 through T-017)

Use `MinimalPlugins` + selected plugins to run game systems in a headless environment.

```
┌────────────────────────────────────┐
│         Test Function              │
│  1. Build App (MinimalPlugins)     │
│  2. Add game plugins/systems       │
│  3. Insert test resources/entities │
│  4. Run app.update() N times       │
│  5. Query world, assert state      │
└────────────────────────────────────┘
```

Tests verify:
- Level spawning creates correct entity counts and component combinations
- Collision between player and destination triggers `GameState::Finished`
- Gravity accelerates player toward planets over multiple frames
- Player clock runs slower than observer clock under time dilation
- `despawn_level` removes all `GameItem` entities

## Phase 5: Headless Render Smoke Test (T-018)

Add at least one integration test that exercises the full Bevy render pipeline without creating an OS window. This replaces the old `cargo run` manual UAT (which required a human to visually verify the game) with an automated test that proves the app can bootstrap and render in CI.

**Approach**: Use `DefaultPlugins` but disable window creation:

```rust
App::new()
    .add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
    )
    .add_plugins(ScheduleRunnerPlugin::run_loop(
        Duration::from_secs_f64(1.0 / 60.0),
    ))
    // ... add game plugins, spawn camera with offscreen RenderTarget::Image
```

The test should:
1. Build the app with `DefaultPlugins` minus window/winit, plus `ScheduleRunnerPlugin`.
2. Add the game's `GamePlugin` (and/or `MenuPlugin`) so the full system graph is exercised.
3. Spawn a camera rendering to an offscreen `Image` asset (not a window surface).
4. Call `app.update()` for several frames.
5. Assert no panics — proving the game initializes, schedules run, and the render graph executes without a display.

This gives CI a meaningful "the game boots and renders" gate without requiring a GPU window or manual verification.

# Assumptions

- Bevy `MinimalPlugins` + `TimePlugin` + `TransformPlugin` provide enough infrastructure for headless ECS tests.
- Bevy `DefaultPlugins` with `WindowPlugin { primary_window: None }` and `WinitPlugin` disabled can execute the render graph headlessly (confirmed supported in Bevy 0.17 via offscreen render targets and `ScheduleRunnerPlugin`).
- The game's physics logic can be cleanly extracted into pure functions without changing system behavior.
- `approx` and `proptest` are compatible with the current nightly Rust toolchain.
- CI runners (GitHub Actions) can run headless Bevy apps without a GPU (the render graph initializes but may use a software/null backend).

# Constraints

- Refactoring must not change observable game behavior — systems must produce identical results after extraction.
- Extracted functions should be `pub(crate)` to avoid expanding the public API.
- ECS-only E2E tests must not require a display server or GPU (no `WindowPlugin`, no `RenderPlugin`).
- The headless render smoke test uses `DefaultPlugins` but disables `WinitPlugin` and sets `primary_window: None` — it must not open an OS window.
- Must work with existing `cargo make test` (nextest) and `cargo make ci` workflows.

# References to Code

- `src/game/shared/helpers.rs` — 7 pure utility functions (collision, coordinate conversion, scaling)
- `src/game/shared/systems.rs` — 7 Bevy systems with embedded physics logic (velocity_update, collision_check, rotation)
- `src/game/shared/types.rs` — Component definitions, `Velocity::scalar()` method
- `src/game/shared/constants.rs` — Physical constants (G, C, masses, velocities)
- `src/game/player/player_clock.rs` — Relativistic gamma calculations (velocity + gravitational time dilation)
- `src/game/player/player_sprite.rs` — Player launch velocity calculation
- `src/game/observer/mod.rs` — Observer clock formatting
- `src/game/levels/mod.rs` — Level spawning and entity configuration
- `src/shared/state.rs` — AppState and GameState enums

# Non-Goals (MVP)

- Visual/screenshot regression testing
- Performance benchmarking (criterion)
- Fuzzing
- Testing menu UI interactions
- Full render output validation (pixel-level correctness) — the headless render smoke test only verifies the app boots and the render graph runs without panics
- 100% test coverage — target is 60–80%
- Testing audio or input handling

# History

## 2026-02-07 — T-001 Completed
- **Task**: Add dev-dependencies: approx and proptest
- **Status**: ✅ Done
- **Changes**:
  - Added `[dev-dependencies]` section to `Cargo.toml` with `approx = "0.5"` and `proptest = "1"`
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 2/2 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-002 Completed
- **Task**: Unit tests for game/shared/helpers.rs (7 pure functions)
- **Status**: ✅ Done
- **Changes**:
  - Added `#[cfg(test)]` module to `src/game/shared/helpers.rs` with 24 unit tests covering all 7 pure functions
  - Tests cover: `has_collided` (overlapping, distant, boundary, same position, symmetry, zero radius, diagonal), `get_translation_from_percentage` (origin, full, half), `get_translation_from_position` (origin, screen edge), `get_position_from_percentage` (origin, full, half), `length_to_pixel` (zero, full width, half width), `planet_sprite_pixel_radius_to_scale` (half sprite width, zero, proportional), `rocket_sprite_pixel_radius_to_scale` (half sprite width, zero, proportional)
  - Used `approx::assert_relative_eq!` for float comparisons
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 26/26 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-003 Completed
- **Task**: Unit tests for game/shared/types.rs (Velocity::scalar)
- **Status**: ✅ Done
- **Changes**:
  - Added `#[cfg(test)]` module to `src/game/shared/types.rs` with 8 unit tests for `Velocity::scalar()`
  - Tests cover: zero velocity, unit x velocity, unit y velocity, Pythagorean triples (3-4-5, 5-12-13), negative components, mixed sign components, non-negativity invariant
  - Used `approx::assert_relative_eq!` for float comparisons and `uom::si::velocity::meter_per_second` for unit-aware construction
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 34/34 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-005 Completed
- **Task**: Extract and test relativistic gamma calculations from player_clock.rs
- **Status**: ✅ Done
- **Changes**:
  - Extracted three pure `pub(crate)` functions from `player_clock_update` in `src/game/player/player_clock.rs`:
    - `calculate_velocity_gamma(vx, vy, c)` — Lorentz gamma factor from velocity
    - `calculate_gravitational_gamma(player_x, player_y, masses)` — gravitational gamma from nearby masses
    - `calculate_player_clock(dt, v_gamma, g_gamma, prev)` — clock advance with time dilation
  - Refactored `player_clock_update` system to call the extracted functions (thin wrapper pattern)
  - Added `#[cfg(test)]` module with 18 unit tests covering:
    - Velocity gamma: at rest == 1, >= 1 always, increases with speed, large near c, symmetric in x/y, sign-independent
    - Gravitational gamma: no masses == 1, >= 1 always, increases closer to mass, increases with mass, compounds across multiple masses, clamps near singularity
    - Player clock: advances at rest, slows with velocity gamma, slows with gravitational gamma, preserves previous value, combined gamma slows more
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 52/52 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-006 Completed
- **Task**: Extract and test gravitational acceleration from game/shared/systems.rs
- **Status**: ✅ Done
- **Changes**:
  - Extracted two pure `pub(crate)` functions from `velocity_update` in `src/game/shared/systems.rs`:
    - `calculate_gravitational_acceleration(pos_x, pos_y, other_pos_x, other_pos_y, other_mass)` — computes gravitational acceleration vector from a single mass, including relativistic adjustment
    - `calculate_relativistic_adjustment(mass, distance)` — computes the relativistic correction factor (0.0–1.0) that reduces acceleration near the Schwarzschild radius
  - Refactored `velocity_update` system to call the extracted functions (thin wrapper pattern)
  - Added `#[cfg(test)]` module with 14 unit tests covering:
    - Relativistic adjustment: near 1.0 far from mass, clamps at zero, between 0 and 1, closer means smaller, more mass means smaller
    - Gravitational acceleration: points toward mass (positive x, negative x, positive y), proportional to mass, inverse-square distance (ratio ≈ 4 for 2x distance), diagonal direction, equal components on 45° diagonal, positive magnitude, zero acceleration when relativistic clamp triggers
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 66/66 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-018 Completed
- **Task**: Headless render smoke test: DefaultPlugins without a window
- **Status**: ✅ Done
- **Changes**:
  - Created `src/lib.rs` to expose `game`, `menu`, and `shared` modules as a library crate, enabling integration tests to import project types
  - Updated `src/main.rs` to import from the library crate (`use relativity::...`) instead of declaring modules directly
  - Created `tests/headless_render_smoke.rs` integration test that:
    - Builds a full Bevy `App` with `DefaultPlugins`, disabling `WinitPlugin` and setting `primary_window: None`
    - Adds `GamePlugin` and `MenuPlugin` (exercising the full system graph)
    - Spawns a `Camera2d` rendering to an offscreen `Image` asset (320×180 `Rgba8UnormSrgb` texture with `RENDER_ATTACHMENT` usage)
    - Calls `app.finish()` and `app.cleanup()` to complete async GPU/plugin initialization
    - Runs 5 `app.update()` cycles and asserts no panics
  - Added nextest timeout override in `.config/nextest.toml` for the headless smoke test (`slow-timeout: period=15s, terminate-after=6`) since GPU initialization under parallel test load can take longer
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 67/67 tests passed ✅
- **Constitution Compliance**: No violations. The `lib.rs` extraction follows the DRY principle (modules defined once, imported by both `main.rs` and integration tests). The `main.rs` change is minimal and preserves identical behavior.

## 2026-02-07 — T-004 Completed
- **Task**: Unit tests for game/shared/constants.rs (value validation)
- **Status**: ✅ Done
- **Changes**:
  - Added `#[cfg(test)]` module to `src/game/shared/constants.rs` with 15 tests (12 runtime + 3 compile-time const assertions)
  - LazyLock initialization tests: verify all 9 `LazyLock` statics (`DAYS_PER_SECOND_UOM`, `UNIT_RADIUS`, `MASS_OF_SUN`, `MASS_OF_EARTH`, `SCREEN_WIDTH_UOM`, `SCREEN_HEIGHT_UOM`, `C`, `G`, `MAX_PLAYER_LAUNCH_VELOCITY`) initialize successfully and produce positive values
  - Physical constant validation: `MASS_OF_SUN > MASS_OF_EARTH`, `MAX_PLAYER_LAUNCH_VELOCITY < C`, max velocity is exactly 99% of C, `SCREEN_HEIGHT < SCREEN_WIDTH` (landscape), `G` matches known gravitational constant value, `C` matches known speed of light
  - Compile-time const assertions for `PLANET_SPRITE_WIDTH_PX > 0`, `ROCKET_SPRITE_WIDTH_PX > 0`, `PLANET_SPRITE_WIDTH_PX > ROCKET_SPRITE_WIDTH_PX`
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 82/82 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-007 Completed
- **Task**: Extract and test rocket rotation calculation from game/shared/systems.rs
- **Status**: ✅ Done
- **Changes**:
  - Extracted pure `pub(crate)` function `calculate_rocket_rotation(vx, vy) -> f32` from `rocket_rotation_update` in `src/game/shared/systems.rs`
  - The function normalizes the velocity vector and computes the rotation angle as `atan2(vy, vx) - π/2`, suitable for `Quat::from_rotation_z`
  - Refactored `rocket_rotation_update` system to call the extracted function (thin wrapper pattern)
  - Added 10 unit tests covering:
    - Cardinal directions: up (0°), right (-π/2), left (+π/2), down (±π)
    - Diagonal directions: up-right (-π/4), up-left (+π/4), down-right (-3π/4), down-left (-5π/4)
    - Magnitude invariance: scaling velocity does not change angle
    - Asymmetric velocity: non-unit vector (3, 4) produces correct angle
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 92/92 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-008 Completed
- **Task**: Extract and test launch velocity calculation from player_sprite.rs
- **Status**: ✅ Done
- **Changes**:
  - Extracted pure `pub(crate)` function `calculate_launch_velocity(cursor_x, cursor_y, player_x, player_y, screen_width_px, max_velocity)` from `player_launch` in `src/game/player/player_sprite.rs`
  - The function computes a launch velocity vector directed from the player toward the cursor, with power scaling linearly with distance (clamped at 80% of screen width)
  - Refactored `player_launch` system to call the extracted function (thin wrapper pattern)
  - Added `#[cfg(test)]` module with 12 unit tests covering:
    - Direction: right, left, up, down, diagonal
    - Power scaling: scales with distance, clamps at max, at exactly 80% threshold
    - Constraints: respects max velocity, very short distance produces small velocity, different max velocity values, horizontal symmetry
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 104/104 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-010 Completed
- **Task**: Add proptest property-based tests for physics invariants
- **Status**: ✅ Done
- **Changes**:
  - Added proptest property-based tests to `src/game/player/player_clock.rs` (nested `proptests` module inside existing `#[cfg(test)]`):
    - `velocity_gamma_ge_one_for_all_sub_c`: verifies γ_v ≥ 1 for arbitrary velocities below c
    - `gravitational_gamma_ge_one_for_positive_mass_and_distance`: verifies γ_g ≥ 1 for arbitrary positive masses and distances
  - Added proptest property-based tests to `src/game/shared/helpers.rs` (nested `proptests` module):
    - `has_collided_is_symmetric_for_all_inputs`: verifies has_collided(a,b) == has_collided(b,a) for arbitrary positions and radii
  - Added proptest property-based tests to `src/game/shared/types.rs` (nested `proptests` module):
    - `scalar_is_non_negative_for_all_inputs`: verifies Velocity::scalar() ≥ 0 for arbitrary velocity components
  - Tests placed inside existing `#[cfg(test)]` modules to access `pub(crate)` functions without changing visibility
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 108/108 tests passed ✅
- **Constitution Compliance**: No violations.

## 2026-02-07 — T-011 Completed
- **Task**: Create test helper module for shared Bevy test setup
- **Status**: ✅ Done
- **Changes**:
  - Created `src/game/test_helpers.rs` with four `pub` helper functions:
    - `minimal_test_app()` — builds a headless Bevy `App` with `MinimalPlugins` + `TransformPlugin`
    - `run_frames(app, n)` — advances the app by N update cycles
    - `spawn_test_entity(world, pos_x, pos_y, vel_x, vel_y, mass, radius)` — spawns an entity with `Position`, `Velocity`, `Mass`, and `Radius` using convenient km/km·s⁻¹/kg units
    - `spawn_positioned_entity(world, pos_x, pos_y)` — spawns an entity at a position with zero velocity/mass/radius
  - Added `#[cfg(test)] pub mod test_helpers` to `src/game/mod.rs`
  - Added 4 unit tests verifying: app creates with `Time` resource, multiple frames run without panic, `spawn_test_entity` populates all components correctly, `spawn_positioned_entity` defaults velocity/mass/radius to zero
  - `cargo make uat` passed: fmt-check ✅, clippy ✅, nextest 112/112 tests passed ✅
- **Constitution Compliance**: No violations.
