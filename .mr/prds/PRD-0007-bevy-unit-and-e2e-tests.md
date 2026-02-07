---
id: PRD-0007
title: "Bevy Unit and E2E Tests"
status: draft
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
    name: "Game still runs correctly after refactoring"
    command: cargo run
    uat_status: unverified
tasks:
  - id: T-001
    title: "Add dev-dependencies: approx and proptest"
    priority: 1
    status: todo
    notes: "Add approx (float comparison) and proptest (property-based testing) to Cargo.toml [dev-dependencies]"
  - id: T-002
    title: "Unit tests for game/shared/helpers.rs (7 pure functions)"
    priority: 1
    status: todo
    notes: "Test has_collided, get_translation_from_position, get_translation_from_percentage, get_position_from_percentage, length_to_pixel, planet_sprite_pixel_radius_to_scale, rocket_sprite_pixel_radius_to_scale with normal, edge, and boundary cases"
  - id: T-003
    title: "Unit tests for game/shared/types.rs (Velocity::scalar)"
    priority: 1
    status: todo
    notes: "Test Velocity::scalar() with zero, unit, and Pythagorean triple inputs"
  - id: T-004
    title: "Unit tests for game/shared/constants.rs (value validation)"
    priority: 2
    status: todo
    notes: "Validate physical constants are reasonable: mass_of_sun > mass_of_earth, max_velocity < C, G and C are positive, LazyLock statics initialize correctly"
  - id: T-005
    title: "Extract and test relativistic gamma calculations from player_clock.rs"
    priority: 1
    status: todo
    notes: "Extract calculate_velocity_gamma, calculate_gravitational_gamma, and calculate_player_clock as pure functions. Test: gamma >= 1 always, gamma == 1 at rest, gamma increases with velocity, gravitational gamma near massive bodies"
  - id: T-006
    title: "Extract and test gravitational acceleration from game/shared/systems.rs"
    priority: 1
    status: todo
    notes: "Extract gravitational acceleration calculation and relativistic velocity adjustment from velocity_update() into pure functions. Test: force proportional to mass, inverse-square distance, direction toward mass"
  - id: T-007
    title: "Extract and test rocket rotation calculation from game/shared/systems.rs"
    priority: 2
    status: todo
    notes: "Extract rotation angle calculation from rocket_rotation_update() into a pure function. Test: angle correct for cardinal and diagonal velocities"
  - id: T-008
    title: "Extract and test launch velocity calculation from player_sprite.rs"
    priority: 2
    status: todo
    notes: "Extract launch velocity/direction calculation from player_launch() into a pure function. Test: respects max velocity, direction toward cursor, power scales with distance"
  - id: T-009
    title: "Extract and test observer clock formatting from observer/mod.rs"
    priority: 3
    status: todo
    notes: "Extract time formatting into a pure function. Test: correct format string output"
  - id: T-010
    title: "Add proptest property-based tests for physics invariants"
    priority: 2
    status: todo
    notes: "Property tests: velocity gamma >= 1 for all velocities < C, gravitational gamma >= 1 for all positive masses/distances, has_collided is symmetric, scalar() >= 0 for all inputs"
  - id: T-011
    title: "Create test helper module for shared Bevy test setup"
    priority: 2
    status: todo
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
---

# Summary

Add comprehensive unit tests and headless E2E tests to the relativity game. This involves extracting pure logic from Bevy systems into standalone testable functions, adding unit tests for all pure game logic (physics, collision, relativity math), and creating headless integration tests using Bevy's `MinimalPlugins` to verify end-to-end game behavior without a GPU or display.

# Problem

The project currently has only 2 trivial unit tests covering enum existence. All core game logic — relativistic time dilation, gravitational physics, collision detection, player movement, and level spawning — is untested. This makes it risky to refactor or add features, and there is no safety net for regressions. The physics calculations are especially critical since the entire game premise depends on correct relativity math.

# Goals

1. Extract pure computation logic from Bevy systems into standalone functions that can be unit tested directly.
2. Achieve 60–80% test coverage across the codebase through unit tests on pure functions.
3. Add property-based tests (via `proptest`) for physics invariants (e.g., gamma ≥ 1, collision symmetry).
4. Create a reusable test helper module for headless Bevy app setup using `MinimalPlugins`.
5. Add E2E headless tests that verify game behavior: level spawning, collision, gravity, time dilation, and cleanup.
6. All tests must run in CI without a GPU or display server.
7. Refactoring must not change any observable game behavior.

# Technical Approach

## Phase 1: Foundation (T-001, T-011)

Add `approx` and `proptest` as dev-dependencies. Create a test helper module with utilities for building headless Bevy test apps.

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

# Assumptions

- Bevy `MinimalPlugins` + `TimePlugin` + `TransformPlugin` provide enough infrastructure for headless ECS tests.
- The game's physics logic can be cleanly extracted into pure functions without changing system behavior.
- `approx` and `proptest` are compatible with the current nightly Rust toolchain.
- CI runners (GitHub Actions) can run headless Bevy apps without a GPU.

# Constraints

- Refactoring must not change observable game behavior — systems must produce identical results after extraction.
- Extracted functions should be `pub(crate)` to avoid expanding the public API.
- E2E tests must not require a display server or GPU (no `WindowPlugin`, no `RenderPlugin`).
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
- Testing asset loading or rendering
- 100% test coverage — target is 60–80%
- Testing audio or input handling

# History
