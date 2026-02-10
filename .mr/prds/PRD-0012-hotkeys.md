---
id: PRD-0012
title: "Hotkeys: Pause, Simulation Speed, and Grid Toggle"
status: active
owner: twitchax
created: 2026-02-10
updated: 2026-02-10
principles:
  - Minimal changes to existing systems; new behavior is additive
  - Use Bevy resources and run conditions rather than modifying core physics math
  - Follow existing HUD patterns (marker components, targeted queries)
  - Hotkeys only affect gameplay state during InGame; no effect on menus
references:
  - name: "Bevy Time / Virtual Time docs"
    url: "https://docs.rs/bevy/latest/bevy/time/struct.Virtual.html"
  - name: "PRD-0010 HUD implementation"
    url: ".mr/prds/PRD-0010-improve-hud.md"
  - name: "PRD-0011 Gravity Grid"
    url: ".mr/prds/PRD-0011-warped-euclidean-gravity-grid.md"
acceptance_tests:
  - id: uat-001
    name: "Pressing Space while Running pauses simulation; pressing again resumes"
    command: cargo make uat
    uat_status: unverified
  - id: uat-002
    name: "Plus key increases sim rate by 0.25x (clamped to 2.00x)"
    command: cargo make uat
    uat_status: unverified
  - id: uat-003
    name: "Minus key decreases sim rate by 0.25x (clamped to 0.25x)"
    command: cargo make uat
    uat_status: unverified
  - id: uat-004
    name: "HUD displays current simulation rate in right panel (always visible)"
    command: cargo make uat
    uat_status: unverified
  - id: uat-005
    name: "Sim rate resets to 1.00x on level start / re-launch"
    command: cargo make uat
    uat_status: unverified
  - id: uat-006
    name: "Pressing G toggles gravity grid visibility on/off"
    command: cargo make uat
    uat_status: unverified
  - id: uat-007
    name: "Speed controls only apply while GameState::Running"
    command: cargo make uat
    uat_status: unverified
tasks:
  - id: T-001
    title: "Add SimPaused game state variant and Space toggle system"
    priority: 1
    status: done
    notes: "Add GameState::SimPaused variant. System listens for Space key, toggles between Running and SimPaused. Physics/clock systems must not run during SimPaused."
  - id: T-002
    title: "Add SimRate resource and +/- hotkey system"
    priority: 1
    status: done
    notes: "SimRate(f64) resource, default 1.0. System listens for +/- (NumpadAdd/NumpadSubtract or Equals/Minus), steps by 0.25, clamps [0.25, 2.00]. Only active when GameState::Running."
  - id: T-003
    title: "Apply SimRate scaling to physics and clock systems"
    priority: 1
    status: done
    notes: "Multiply time.delta_secs() by SimRate in velocity_update, position_update, player_clock_system, and observer_clock_system. Alternatively, use Bevy Virtual time relative_speed."
  - id: T-004
    title: "Reset SimRate to 1.0 on level start / re-launch"
    priority: 2
    status: done
    notes: "Reset in spawn_level or when transitioning to GameState::Running from Paused (launch fire)."
  - id: T-005
    title: "Add sim rate HUD label in right (observer) panel"
    priority: 2
    status: done
    notes: "New HudSimRate marker component. Display as 'r = 1.00×'. Spawn in observer panel labels. Add update system."
  - id: T-006
    title: "Add GridVisible resource and G toggle system"
    priority: 2
    status: done
    notes: "GridVisible(bool) resource, default true. System listens for G key, toggles value. gravity_grid_render_system early-returns when GridVisible is false."
  - id: T-007
    title: "Verify clippy, fmt, and existing tests pass"
    priority: 3
    status: done
    notes: "Run cargo make ci to ensure no regressions."
---

# Summary

Add keyboard hotkeys for pausing the simulation (Space), adjusting simulation speed (+/− keys with a HUD rate indicator), and toggling the gravity grid (G). These controls give players finer control over the game experience and aid in understanding relativistic effects by letting them slow down or speed up time.

# Problem

The game currently has no way to pause mid-flight, adjust simulation speed, or hide the gravity grid. Players watching relativistic effects unfold have no control over pacing, and the grid (while helpful) can be visually distracting. These are standard quality-of-life controls expected in simulation-style games.

# Goals

1. **Pause/Resume** — Space key toggles between running and paused states during flight, freezing all physics and clocks.
2. **Simulation Speed** — +/− keys adjust a speed multiplier from 0.25× to 2.00× in 0.25× increments, applied to all time-dependent systems.
3. **Rate HUD** — A persistent indicator in the right HUD panel shows the current simulation rate (e.g., `r = 1.00×`).
4. **Grid Toggle** — G key shows/hides the gravity grid.

# Technical Approach

## Pause (Space)

Add a `GameState::SimPaused` variant. A new `sim_pause_toggle` system (running in `AppState::InGame`) listens for `KeyCode::Space` `just_pressed` and toggles between `GameState::Running` and `GameState::SimPaused`. All physics and clock update systems already gate on `GameState::Running`, so they will automatically freeze when `SimPaused` is active.

```
Running ──[Space]──▶ SimPaused
SimPaused ──[Space]──▶ Running
```

## Simulation Speed (+/−)

Introduce a `SimRate` resource (`f64`, default 1.0). A `sim_rate_adjust` system listens for `+`/`-` keys (both numpad and standard keyboard) and steps by 0.25, clamping to `[0.25, 2.00]`. The preferred approach is to use Bevy's `Time<Virtual>` `relative_speed` so all systems that read `Time` automatically respect the rate without manual multiplication. If `Time<Virtual>` is not viable (e.g., it affects menu transitions), then manually multiply `time.delta_secs()` by `sim_rate.0` in the four core systems: `velocity_update`, `position_update`, `player_clock_system`, `observer_clock_system`.

The rate resets to 1.0 when the player fires (transition from `Paused` → `Running` in `launch_fire_system`).

## Rate HUD

Add a `HudSimRate` marker component. Spawn a new label in the observer panel (`spawn_observer_labels`) with initial text `r = 1.00×`. An `sim_rate_hud_update` system reads `Res<SimRate>` and updates the text each frame.

## Grid Toggle (G)

Add a `GridVisible(bool)` resource (default `true`). A `grid_toggle` system listens for `KeyCode::KeyG` `just_pressed` and flips the bool. The existing `gravity_grid_render_system` adds an early return when `GridVisible` is `false`.

# Assumptions

- Bevy's `Time<Virtual>` `relative_speed` is available and suitable for scaling game time without affecting UI/menu transitions. If not, manual delta scaling is the fallback.
- The existing `GameState` enum can accommodate a new `SimPaused` variant without breaking state transition logic.
- The observer panel in the HUD has room for one additional label.

# Constraints

- Pause and speed controls must not interfere with the Escape-to-menu flow.
- Speed changes must not apply during the launch/aim phase (`GameState::Paused`).
- The grid toggle must not affect grid state across level reloads (reset to default on level spawn).
- All changes must pass `cargo make ci` (clippy pedantic, fmt, nextest).

# References to Code

- `src/game/shared/systems.rs` — `velocity_update`, `position_update`, `exit_level_check` (keyboard input pattern)
- `src/game/player/player_clock.rs` — `player_clock_system` (time-dependent)
- `src/game/player/player_sprite.rs` — `launch_fire_system` (state transitions)
- `src/game/hud/mod.rs` — HUD spawn and update systems, marker components
- `src/game/gravity_grid/mod.rs` — `gravity_grid_render_system` (grid rendering)
- `src/game/mod.rs` — System registration and run conditions
- `src/shared/state.rs` — `GameState`, `AppState` enums

# Non-Goals (MVP)

- Pause overlay or pause menu UI (just freeze the simulation)
- Configurable keybindings
- Speed values outside 0.25×–2.00× range
- Per-system speed control (all systems use the same rate)
- Persisting grid visibility preference across sessions

# History

## 2026-02-10 — T-001 Completed
- **Task**: Add SimPaused game state variant and Space toggle system
- **Status**: ✅ Done
- **Changes**:
  - Added `GameState::SimPaused` variant to `src/shared/state.rs`
  - Added `sim_pause_toggle` system in `src/game/shared/systems.rs` — listens for `KeyCode::Space` and toggles between `Running` and `SimPaused`
  - Registered `sim_pause_toggle` in `GamePlugin` (`src/game/mod.rs`) to run during `AppState::InGame` (all sub-states)
  - Updated unit tests in `state.rs` for the new variant (distinctness, debug format)
  - Physics/clock systems already gate on `GameState::Running`, so they automatically freeze during `SimPaused`
  - UAT: `cargo make uat` passed — 258 tests, 258 passed, 0 skipped
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-002 Completed
- **Task**: Add SimRate resource and +/- hotkey system
- **Status**: ✅ Done
- **Changes**:
  - Added `SimRate` resource to `src/game/shared/types.rs` — `SimRate(f64)` with `Default` (1.0), constants `MIN` (0.25), `MAX` (2.0), `STEP` (0.25)
  - Added `sim_rate_adjust` system in `src/game/shared/systems.rs` — listens for `NumpadAdd`/`Equal` (increase) and `NumpadSubtract`/`Minus` (decrease), steps by 0.25, clamps to [0.25, 2.00]
  - Registered `SimRate` as init resource in `GamePlugin` (`src/game/mod.rs`)
  - Registered `sim_rate_adjust` in the `GameState::Running` system set so it only applies while running
  - Added unit tests for `SimRate` default value and constants in `types.rs`
  - UAT: `cargo make uat` passed — 260 tests, 260 passed, 0 skipped
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-003 Completed
- **Task**: Apply SimRate scaling to physics and clock systems
- **Status**: ✅ Done
- **Changes**:
  - Added `Res<SimRate>` parameter to `velocity_update` and `position_update` in `src/game/shared/systems.rs` — time delta is now multiplied by `sim_rate.0`
  - Added `Res<SimRate>` parameter to `player_clock_update` in `src/game/player/player_clock.rs` — added `SimRate` import and scaled time delta
  - Added `Res<SimRate>` parameter to `observer_clock_update` in `src/game/observer/mod.rs` — added `SimRate` import and scaled time delta
  - Updated `tests/e2e_time_dilation.rs` — added `SimRate` import and `.init_resource::<SimRate>()` to test app builder
  - Updated `tests/e2e_velocity_update.rs` — added `SimRate` import and `.init_resource::<SimRate>()` to test app builder
  - UAT: `cargo make uat` passed — 260 tests, 260 passed, 0 skipped
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-004 Completed
- **Task**: Reset SimRate to 1.0 on level start / re-launch
- **Status**: ✅ Done
- **Changes**:
  - Added `reset_sim_rate` system in `src/game/shared/systems.rs` — resets `SimRate` to 1.0
  - Registered `reset_sim_rate` on `OnEnter(AppState::InGame)` in `src/game/mod.rs` alongside `spawn_level`
  - Added `SimRate` reset in `launch_fire_system` (`src/game/player/player_sprite.rs`) — resets to 1.0 when transitioning from `Paused` → `Running`
  - Added `SimRate` reset in `reset_level_on_pending` (`src/game/levels/mod.rs`) — resets to 1.0 on level reset after failure
  - UAT: `cargo make uat` passed — 260 tests, 260 passed, 0 skipped
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-005 Completed
- **Task**: Add sim rate HUD label in right (observer) panel
- **Status**: ✅ Done
- **Changes**:
  - Added `HudSimRate` marker component in `src/game/hud/mod.rs`
  - Spawned `r = 1.00×` label in `spawn_observer_labels` (observer panel), repositioning existing `t_o` label from 50% to 35% vertical and placing sim rate at 65% for balanced layout
  - Added `sim_rate_hud_update` system that reads `Res<SimRate>` and updates the `HudSimRate` text each frame
  - Imported `SimRate` in `src/game/hud/mod.rs`
  - Registered `sim_rate_hud_update` in `GamePlugin` (`src/game/mod.rs`) to run during `AppState::InGame` (all sub-states, so rate is visible even when paused)
  - Exported `sim_rate_hud_update` from `hud` module and imported in `game/mod.rs`
  - UAT: `cargo make uat` passed — 260 tests, 260 passed, 0 skipped
  - Opportunistic UAT check: uat-004 ("HUD displays current simulation rate in right panel") is now implementable but deferred to the UAT verification loop
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-006 Completed
- **Task**: Add GridVisible resource and G toggle system
- **Status**: ✅ Done
- **Changes**:
  - Added `GridVisible(bool)` resource to `src/game/shared/types.rs` — default `true`, with unit test
  - Added `grid_toggle` system in `src/game/shared/systems.rs` — listens for `KeyCode::KeyG` and flips `GridVisible`
  - Added `reset_grid_visible` system in `src/game/shared/systems.rs` — resets to `true`
  - Updated `gravity_grid_render_system` in `src/game/gravity_grid/mod.rs` — accepts `Res<GridVisible>`, early-returns when `false`
  - Registered `GridVisible` as init resource in `GamePlugin` (`src/game/mod.rs`)
  - Registered `grid_toggle` in `AppState::InGame` system set (works across all sub-states)
  - Registered `reset_grid_visible` on `OnEnter(AppState::InGame)` alongside `reset_sim_rate`
  - Added `GridVisible` reset in `reset_level_on_pending` (`src/game/levels/mod.rs`) with `#[allow(clippy::too_many_arguments)]`
  - UAT: `cargo make uat` passed — 261 tests, 261 passed, 0 skipped
- **Constitution Compliance**: No violations.

## 2026-02-10 — T-007 Completed
- **Task**: Verify clippy, fmt, and existing tests pass
- **Status**: ✅ Done
- **Changes**:
  - Ran `cargo make ci` (fmt-check + clippy + nextest)
  - All checks passed: 261 tests run, 261 passed, 0 skipped
  - No code changes required — all previous tasks left the codebase in a clean state
- **Constitution Compliance**: No violations.
