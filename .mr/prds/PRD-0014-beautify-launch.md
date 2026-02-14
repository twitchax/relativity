---
id: PRD-0014
title: "Beautify Launch: Intuitive Aim Preview and Radial Power Arc"
status: active
owner: twitchax
created: 2026-02-14
updated: 2026-02-14
principles:
  - "Launch interaction should feel intuitive without instructions"
  - "Power feedback should be spatially co-located with the player, not a separate HUD element"
  - "Preview aim direction on hover so the player can plan before committing"
  - "Non-linear power curve gives fine control at low speeds and dramatic feel at high speeds"
  - "Minimal changes to existing state machine and pure functions"
references:
  - name: "PRD-0009: Improve Visuals (original launch UX)"
    url: ".mr/prds/PRD-0009-improve-visuals.md"
  - name: "Bevy Gizmos documentation"
    url: "https://docs.rs/bevy/latest/bevy/gizmos/index.html"
acceptance_tests:
  - id: uat-001
    name: "Dotted aim preview line renders on hover before any click"
    command: cargo make uat
    uat_status: verified
  - id: uat-002
    name: "Click locks aim direction and transitions LaunchState to AimLocked"
    command: cargo make uat
    uat_status: verified
  - id: uat-003
    name: "Drag along aim direction fills radial arc around player with correct color gradient"
    command: cargo make uat
    uat_status: verified
  - id: uat-004
    name: "Arc tick marks appear at 0.25c, 0.5c, 0.75c, and 0.9c"
    command: cargo make uat
    uat_status: verified
  - id: uat-005
    name: "Numeric velocity readout (e.g. 0.45c) displays near the arc during Launching phase"
    command: cargo make uat
    uat_status: verified
  - id: uat-006
    name: "Release fires player at correct angle and power; GameState transitions to Running"
    command: cargo make uat
    uat_status: verified
  - id: uat-007
    name: "Right-click or Escape cancels launch at any phase and returns to Idle"
    command: cargo make uat
    uat_status: verified
  - id: uat-008
    name: "Power maps to 0.1c–0.99c range with non-linear (exponential ease-in) curve"
    command: cargo make uat
    uat_status: verified
  - id: uat-009
    name: "Old bottom-center power bar UI no longer appears"
    command: cargo make uat
    uat_status: verified
  - id: uat-010
    name: "All existing launch unit tests pass or are updated to match new behavior"
    command: cargo make test
    uat_status: verified
tasks:
  - id: T-001
    title: "Add hover preview system (dotted aim line on mouse move, before click)"
    priority: 1
    status: done
    notes: "New system `launch_preview_system` runs during GameState::Paused + LaunchState::Idle. Renders a thin dotted line from player toward cursor using Gizmos. Runs every frame on cursor move."
  - id: T-002
    title: "Add right-click / Escape cancel support to launch systems"
    priority: 1
    status: done
    notes: "New system `launch_cancel_system` resets LaunchState to Idle on right-click or Escape from any non-Idle state."
  - id: T-003
    title: "Replace power bar with radial arc around player"
    priority: 1
    status: done
    notes: "Remove `spawn_or_update_power_bar` and `PowerBarUi`. In `launch_visual_system`, draw a radial arc (Gizmos::arc_2d) centered on the player. Arc spans 0–270° proportional to power. Color gradient: cyan at 0.1c → orange → red at 0.99c."
  - id: T-004
    title: "Add tick marks on the arc at 0.25c, 0.5c, 0.75c, 0.9c"
    priority: 2
    status: done
    notes: "Short radial lines at fixed angular positions corresponding to those velocities. Rendered via Gizmos during Launching phase."
  - id: T-005
    title: "Add numeric velocity readout near the arc"
    priority: 2
    status: done
    notes: "Spawn a small text entity (or use Gizmos text if available) near the arc tip showing e.g. '0.45c'. Update each frame during Launching. Despawn on Idle."
  - id: T-006
    title: "Implement non-linear power curve (exponential ease-in, 0.1c–0.99c)"
    priority: 2
    status: done
    notes: "Replace linear power mapping. Raw power 0.0–1.0 from drag → apply exponential ease-in → map to 0.1c–0.99c. Update `calculate_launch_velocity_from_angle_power` or add a new mapping function. Minimum launch velocity is 0.1c."
  - id: T-007
    title: "Update launch_visual_system to use new arc + dotted line visuals"
    priority: 1
    status: done
    notes: "Refactor `launch_visual_system` to render: AimLocked → solid direction line + empty arc outline; Launching → solid direction line (length scaled by power) + filled arc + tick marks + readout."
  - id: T-008
    title: "Update and add unit tests for new power curve and arc rendering logic"
    priority: 3
    status: done
    notes: "Update existing tests for `calculate_launch_velocity_from_angle_power`. Add tests for non-linear mapping, 0.1c minimum, cancel behavior."
  - id: T-009
    title: "Clean up dead code (PowerBarUi, spawn_or_update_power_bar)"
    priority: 3
    status: done
    notes: "Remove PowerBarUi component from types.rs, remove spawn_or_update_power_bar from player_sprite.rs, remove any references in mod.rs."
---

# Summary

Replace the current launch system's disconnected power bar and basic gizmo line with an intuitive aim-preview-on-hover, radial power arc around the player, and non-linear velocity curve. The new interface provides spatial feedback co-located with the player, a dotted preview line before committing, tick marks at notable relativistic velocities, and a numeric readout — making it easy to reason about launch power in a game centered on relativity.

# Problem

The current launch system has three usability issues:

1. **No preview before click**: The player cannot see where they're aiming until they click, forcing a blind commitment.
2. **Disconnected power bar**: The 204px power bar at bottom-center of the screen is far from the player and hard to glance at while also watching aim direction.
3. **Unintuitive power mapping**: Drag distance from the player in any direction increases power equally, with no spatial relationship between gesture direction and result. The linear 0–1.0 mapping also makes it hard to fine-tune at low velocities.

# Goals

1. Show a dotted aim preview line on hover (before any click) so the player can plan their shot.
2. Replace the bottom-center power bar with a radial arc centered on the player, providing at-a-glance power feedback where the player is already looking.
3. Color-code the arc from cool (cyan, low velocity) to hot (red, near light speed).
4. Add tick marks at 0.25c, 0.5c, 0.75c, and 0.9c to help target specific relativistic regimes.
5. Display a numeric velocity readout (e.g., "0.45c") near the arc during the power phase.
6. Implement a non-linear power curve (exponential ease-in) mapping to 0.1c–0.99c, giving fine control at low speeds.
7. Add right-click / Escape cancel support from any launch phase.

# Technical Approach

## State Machine (unchanged)

The existing `LaunchState` enum remains:

```
Idle → (click) → AimLocked { angle } → (drag) → Launching { angle, power } → (release) → Idle
                                                                              → (cancel) → Idle
```

A new cancel path is added from any non-Idle state back to Idle via right-click or Escape.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  GameState::Paused systems (run every frame)            │
│                                                         │
│  ┌───────────────────┐   ┌──────────────────────┐       │
│  │ launch_preview    │   │ launch_aim_system     │       │
│  │ (hover dotted     │   │ (click → AimLocked)   │       │
│  │  line, Idle only) │   │                        │       │
│  └───────────────────┘   └──────────────────────┘       │
│                                                         │
│  ┌───────────────────┐   ┌──────────────────────┐       │
│  │ launch_power      │   │ launch_fire_system    │       │
│  │ (drag → power)    │   │ (release → fire)      │       │
│  └───────────────────┘   └──────────────────────┘       │
│                                                         │
│  ┌───────────────────┐   ┌──────────────────────┐       │
│  │ launch_cancel     │   │ launch_visual_system  │       │
│  │ (right-click/Esc  │   │ (arc + line + ticks   │       │
│  │  → Idle)          │   │  + readout)            │       │
│  └───────────────────┘   └──────────────────────┘       │
└─────────────────────────────────────────────────────────┘
```

### New: `launch_preview_system`

- Runs only when `LaunchState::Idle` and `GameState::Paused`.
- Reads cursor position each frame and draws a thin dotted line from the player toward the cursor using `Gizmos`.
- Low-opacity white color to distinguish from the committed aim line.

### New: `launch_cancel_system`

- Listens for `MouseButton::Right` just-pressed or `KeyCode::Escape` just-pressed.
- If `LaunchState` is not `Idle`, resets it to `Idle`.

### Modified: `launch_visual_system`

- **AimLocked**: Solid direction line + faint arc outline (empty, showing max range).
- **Launching**: Solid direction line (length scaled by power) + filled radial arc + tick marks + numeric readout.
- Arc is drawn with `Gizmos::arc_2d` centered on the player, spanning 0–270° proportional to normalized power.
- Color gradient computed from power: `cyan (0.1c) → yellow (0.5c) → orange (0.75c) → red (0.99c)`.
- Tick marks: short radial line segments at angular positions corresponding to 0.25c, 0.5c, 0.75c, 0.9c.
- Velocity text: a small UI text entity or 2D text near the arc tip.

### Modified: `calculate_launch_velocity_from_angle_power` (or new helper)

- Non-linear mapping: `mapped_power = 0.1 + 0.89 * raw_power^2` (quadratic ease-in).
- This maps raw drag power (0.0–1.0) to velocity fraction (0.1c–0.99c).
- Fine-grained control in the 0.1c–0.5c range; rapid scaling above 0.75c.

### Removed

- `PowerBarUi` component.
- `spawn_or_update_power_bar` function.

# Assumptions

- Bevy `Gizmos` API supports `arc_2d` or equivalent for drawing arcs (available since Bevy 0.14).
- The dotted-line effect can be approximated with multiple short line segments or dashes via Gizmos.
- The radial arc radius is a fixed pixel size (not world-scale), so it remains readable at any zoom level.

# Constraints

- Must not change the `LaunchState` enum variants or their semantics (only add the cancel path).
- Must preserve the existing `calculate_launch_velocity_from_angle_power` function signature for backward compatibility, or update all call sites if the signature changes.
- Mouse-only input; no touch/gamepad support in this PRD.
- The arc and preview line must not interfere with the HUD (bevy_lunex) or the gravity grid overlay.

# References to Code

- `src/game/player/player_sprite.rs` — All four launch systems, `spawn_or_update_power_bar`, `calculate_launch_velocity_from_angle_power`.
- `src/game/shared/types.rs` — `LaunchState` enum, `PowerBarUi` marker component.
- `src/game/shared/constants.rs` — `MAX_PLAYER_LAUNCH_VELOCITY`, `C` (speed of light).
- `src/game/mod.rs:29,93` — System registration in `GamePlugin`.
- `src/shared/mod.rs` — `SCREEN_WIDTH_PX`, `SCREEN_HEIGHT_PX`.

# Non-Goals (MVP)

- Touch / gamepad input support.
- Animated arc fill (instant update each frame is fine).
- Sound effects on launch or power changes.
- Trajectory prediction line showing the flight path.
- Changing the `LaunchState` enum variant structure (e.g., adding new states).

# History

## 2026-02-14 — T-001 Completed
- **Task**: Add hover preview system (dotted aim line on mouse move, before click)
- **Status**: ✅ Done
- **Changes**:
  - Added `launch_preview_system` in `src/game/player/player_sprite.rs` — draws a dotted (dashed) aim-preview line from the player toward the cursor using Gizmos while `LaunchState::Idle`.
  - Registered `launch_preview_system` in the Paused launch systems tuple in `src/game/mod.rs`.
  - Added constants `PREVIEW_LINE_LENGTH`, `PREVIEW_DASH_LENGTH`, `PREVIEW_GAP_LENGTH` for the dash pattern.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-002 Completed
- **Task**: Add right-click / Escape cancel support to launch systems
- **Status**: ✅ Done
- **Changes**:
  - Added `launch_cancel_system` in `src/game/player/player_sprite.rs` — listens for `MouseButton::Right` or `KeyCode::Escape` just-pressed and resets `LaunchState` to `Idle` from any non-Idle state.
  - Registered `launch_cancel_system` in the Paused launch systems tuple in `src/game/mod.rs`.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-003 Completed
- **Task**: Replace power bar with radial arc around player
- **Status**: ✅ Done
- **Changes**:
  - Replaced `spawn_or_update_power_bar` and `PowerBarUi` usage in `launch_visual_system` (`src/game/player/player_sprite.rs`) with a radial arc drawn via `Gizmos::arc_2d`.
  - Added constants `ARC_RADIUS` (50px) and `MAX_ARC_ANGLE` (270°) for arc geometry.
  - Added `power_to_color` helper function implementing cyan → orange → red color gradient.
  - AimLocked state now shows a faint arc outline (0.15 alpha) in addition to the direction line.
  - Launching state shows the faint outline plus a filled arc proportional to power with the color gradient.
  - Removed `PowerBarUi` import and `Commands`/query parameters from `launch_visual_system` (no more UI entity spawning).
  - Updated `tests/e2e_launch_visuals.rs`: replaced PowerBarUi entity-count assertions with gizmo-path smoke tests (system runs without panic in each state).
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-007 Completed
- **Task**: Update launch_visual_system to use new arc + dotted line visuals
- **Status**: ✅ Done
- **Changes**:
  - Extracted `draw_dashed_line` helper function in `src/game/player/player_sprite.rs` (DRY: reused by `launch_preview_system` and `launch_visual_system`).
  - Renamed constants `PREVIEW_DASH_LENGTH`/`PREVIEW_GAP_LENGTH` to `DASH_LENGTH`/`DASH_GAP` since they are now shared.
  - Added named constants `AIM_LINE_LENGTH` (200px), `MIN_LAUNCH_LINE` (100px), `MAX_LAUNCH_LINE` (300px) replacing magic numbers.
  - AimLocked state now draws a dotted extension line beyond the solid aim line, providing visual continuity with the idle preview.
  - Launching state now draws a dotted extension beyond the power-scaled solid line, showing remaining potential range.
  - Refactored `launch_preview_system` to use the shared `draw_dashed_line` helper.
  - Enhanced doc comment on `launch_visual_system` documenting per-state rendering behavior.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-004 Completed
- **Task**: Add tick marks on the arc at 0.25c, 0.5c, 0.75c, 0.9c
- **Status**: ✅ Done
- **Changes**:
  - Added constants `TICK_HALF_LENGTH` (6px) and `TICK_VELOCITY_FRACTIONS` ([0.25, 0.5, 0.75, 0.9]) in `src/game/player/player_sprite.rs`.
  - Added `draw_arc_ticks` helper function that draws short radial line segments at angular positions corresponding to each velocity fraction on the arc.
  - Called `draw_arc_ticks` from the `Launching` arm of `launch_visual_system`.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-005 Completed
- **Task**: Add numeric velocity readout near the arc
- **Status**: ✅ Done
- **Changes**:
  - Added `VelocityReadout` marker component in `src/game/shared/types.rs`.
  - Added `launch_readout_system` in `src/game/player/player_sprite.rs` — spawns a `Text2d` entity near the arc tip during `Launching` phase, showing the velocity as a fraction of c (e.g. "0.45c"). Despawns when not launching.
  - Added constants `READOUT_OFFSET` (18px) and `READOUT_FONT_SIZE` (14px) for readout positioning and styling.
  - Registered `launch_readout_system` in the Paused launch systems tuple in `src/game/mod.rs`.
  - The readout uses `Orbitron-Regular.ttf` font and is color-matched to the power arc via `power_to_color`.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-006 Completed
- **Task**: Implement non-linear power curve (exponential ease-in, 0.1c–0.99c)
- **Status**: ✅ Done
- **Changes**:
  - Added `map_power_nonlinear` helper function in `src/game/player/player_sprite.rs` — quadratic ease-in mapping raw power (0.0–1.0) to effective power fraction (≈0.101–1.0), giving velocity range 0.1c–0.99c.
  - Added `MIN_POWER_FRACTION` constant (0.1/0.99 ≈ 0.101) for the minimum velocity floor.
  - Updated `calculate_launch_velocity_from_angle_power` to use the non-linear mapping instead of a linear clamp.
  - Updated `launch_visual_system` to use mapped power for arc fill angle, direction line length, and color gradient.
  - Updated `launch_readout_system` to display velocity based on mapped power and position the readout using mapped arc fill angle.
  - Updated `draw_arc_ticks` comment to reflect the non-linear mapping context.
  - Updated 3 unit tests that relied on linear power behavior: `angle_power_zero_power_produces_minimum_velocity`, `angle_power_clamped_at_max`, `angle_power_nonlinear_half_power_slower_than_linear`.
  - UAT: `cargo make uat` passed — 291 tests, 291 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — T-008 Completed
- **Task**: Update and add unit tests for new power curve and arc rendering logic
- **Status**: ✅ Done
- **Changes**:
  - Added 6 unit tests for `map_power_nonlinear` in `src/game/player/player_sprite.rs`: boundary values (0.0 → MIN_POWER_FRACTION, 1.0 → 1.0), clamping (negative, >1.0), monotonicity, and below-linear-reference at quarter power.
  - Added 4 unit tests for `power_to_color` in `src/game/player/player_sprite.rs`: cyan at 0.0, red at 1.0, smooth transition at 0.5, and out-of-range clamping safety.
  - Changed `power_to_color` visibility from `fn` to `pub(crate) fn` to enable testing.
  - Created `tests/e2e_launch_cancel.rs` with 5 tests: cancel from AimLocked via right-click and Escape, cancel from Launching via right-click and Escape, and no-op when already Idle.
  - UAT: `cargo make uat` passed — 306 tests, 306 passed, 0 skipped.
- **Constitution Compliance**: No violations. Changed `power_to_color` from private to `pub(crate)` — minimal scope widening to enable testing, no public API change.

## 2026-02-14 — T-009 Completed
- **Task**: Clean up dead code (PowerBarUi, spawn_or_update_power_bar)
- **Status**: ✅ Done
- **Changes**:
  - Removed `PowerBarUi` marker component from `src/game/shared/types.rs` (struct and doc comment).
  - Updated `LaunchState` doc comment to say "radial arc visible" instead of "power bar UI visible".
  - `spawn_or_update_power_bar` was already removed in T-003; no further action needed.
  - No references to either symbol remained in source or test code.
  - UAT: `cargo make uat` passed — 306 tests, 306 passed, 0 skipped.
- **Constitution Compliance**: No violations.

## 2026-02-14 — uat-001 Verification
- **UAT**: Dotted aim preview line renders on hover before any click
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_launch_preview.rs` with two tests:
    - `preview_line_draws_on_hover_in_idle`: spawns a headless `PrimaryWindow` with injected cursor position and a `Camera2d` with pre-computed projection values, then runs `launch_preview_system` via `run_system_once` — exercises the full `draw_dashed_line` code path without panicking.
    - `preview_line_skipped_when_not_idle`: confirms the system early-returns (no drawing) in `AimLocked` and `Launching` states.
  - Key technique: Bevy 0.18 `Window::set_physical_cursor_position` and manual `Camera.computed` setup allow `viewport_to_world_2d` to succeed in headless tests.
  - `cargo make uat` passed — 308 tests, 308 passed, 0 skipped.

## 2026-02-14 — uat-002 Verification
- **UAT**: Click locks aim direction and transitions LaunchState to AimLocked
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_launch_aim_lock.rs` with two tests:
    - `click_locks_aim_direction`: spawns a headless window with injected cursor position and camera, simulates a left-click via `ButtonInput<MouseButton>`, runs `launch_aim_system` via `run_system_once`, and asserts LaunchState transitions from Idle to AimLocked with a finite aim angle.
    - `click_does_not_re_aim_when_already_locked`: confirms the system early-returns when LaunchState is already AimLocked, preserving the original angle.
  - `cargo make uat` passed — 310 tests, 310 passed, 0 skipped.

## 2026-02-14 — uat-003 Verification
- **UAT**: Drag along aim direction fills radial arc around player with correct color gradient
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_launch_arc_gradient.rs` with three tests:
    - `arc_fills_at_increasing_power_levels_without_panic`: runs `launch_visual_system` in Launching state at five power levels (0.0–1.0), exercising the filled-arc + color gradient code path.
    - `arc_fill_fraction_increases_with_power`: asserts `map_power_nonlinear` returns monotonically increasing fill fractions across the power sweep.
    - `color_gradient_transitions_cyan_to_red`: asserts the color gradient via `power_to_color` transitions from cyan (low power) to red (full power) with non-decreasing red channel.
  - Widened `map_power_nonlinear` and `power_to_color` from `pub(crate)` to `pub` (with `#[must_use]`) so integration tests can call them directly.
  - `cargo make uat` passed — 313 tests, 313 passed, 0 skipped.

## 2026-02-14 — uat-004 Verification
- **UAT**: Arc tick marks appear at 0.25c, 0.5c, 0.75c, and 0.9c
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_launch_arc_ticks.rs` with four tests:
    - `tick_velocity_fractions_match_spec`: asserts `TICK_VELOCITY_FRACTIONS` contains exactly [0.25, 0.5, 0.75, 0.9].
    - `tick_angular_positions_within_arc_range`: verifies each tick's angular position falls within the arc's sweep range.
    - `tick_marks_drawn_during_launching_without_panic`: runs `launch_visual_system` in Launching state at five power levels, exercising the `draw_arc_ticks` code path.
    - `tick_fractions_produce_valid_mapped_powers`: confirms each tick fraction produces a monotonically increasing mapped power within [0, 1].
  - Widened `TICK_VELOCITY_FRACTIONS` and `MAX_ARC_ANGLE` from private to `pub` so integration tests can verify the constants directly.
  - `cargo make uat` passed — 317 tests, 317 passed, 0 skipped.

## 2026-02-14 — uat-005 Verification
- **UAT**: Numeric velocity readout (e.g. 0.45c) displays near the arc during Launching phase
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_launch_readout.rs` with three tests:
    - `readout_spawns_with_correct_text_during_launching`: enters Launching state, runs `launch_readout_system`, and asserts a `VelocityReadout` entity is spawned with text matching the expected "X.XXc" format derived from `map_power_nonlinear`.
    - `readout_despawns_when_idle`: spawns the readout in Launching, then returns to Idle and verifies the `VelocityReadout` entity is despawned.
    - `readout_updates_on_power_change`: spawns at low power, increases power, and verifies the text content updates to reflect the new velocity fraction.
  - `cargo make uat` passed — 320 tests, 320 passed, 0 skipped.

## 2026-02-14 — uat-006 Verification
- **UAT**: Release fires player at correct angle and power; GameState transitions to Running
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - `tests/e2e_launch_state_machine.rs::launching_to_running_via_fire_system` (lines 94–141) already covers this acceptance criterion:
    - Sets LaunchState to Launching with angle=π/4 (45°) and power=0.8
    - Injects mouse release via `ButtonInput<MouseButton>`
    - Runs `launch_fire_system` via `run_system_once`
    - Asserts LaunchState resets to Idle
    - Asserts GameState transitions to Running
    - Asserts player velocity vx>0, vy>0 (correct angle), and vx/vy ≈ 1.0 (45° symmetry)
  - `tests/e2e_launch_state_machine.rs::full_launch_state_machine_idle_to_running` (lines 176–206) provides additional coverage with a complete round-trip (Idle → AimLocked → Launching → Running).
  - `cargo make uat` passed — 320 tests, 320 passed, 0 skipped.

## 2026-02-14 — uat-007 Verification
- **UAT**: Right-click or Escape cancels launch at any phase and returns to Idle
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - `tests/e2e_launch_cancel.rs` already contains 5 tests covering this criterion:
    - `cancel_aim_locked_via_right_click`: right-click during AimLocked → Idle
    - `cancel_aim_locked_via_escape`: Escape during AimLocked → Idle
    - `cancel_launching_via_right_click`: right-click during Launching → Idle
    - `cancel_launching_via_escape`: Escape during Launching → Idle
    - `cancel_noop_when_idle`: cancel is no-op when already Idle
  - All tests exercise `launch_cancel_system` via `run_system_once` and assert `LaunchState` resets to `Idle`.
  - `cargo make uat` passed — 320 tests, 320 passed, 0 skipped.

## 2026-02-14 — uat-008 Verification
- **UAT**: Power maps to 0.1c–0.99c range with non-linear (exponential ease-in) curve
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - 6 unit tests in `src/game/player/player_sprite.rs` directly test `map_power_nonlinear`:
    - `nonlinear_zero_gives_min_fraction`: verifies `map_power_nonlinear(0.0) ≈ MIN_POWER_FRACTION` (0.1/0.99 ≈ 0.101), mapping to 0.1c
    - `nonlinear_one_gives_one`: verifies `map_power_nonlinear(1.0) = 1.0`, mapping to 0.99c
    - `nonlinear_clamps_negative` / `nonlinear_clamps_above_one`: boundary clamping
    - `nonlinear_monotonically_increasing`: confirms ease-in curve is strictly increasing
    - `nonlinear_quarter_power_below_linear`: confirms quadratic curve yields less than linear at 0.25 (ease-in characteristic)
  - 3 integration tests in `src/game/player/player_sprite.rs` verify end-to-end through `calculate_launch_velocity_from_angle_power`:
    - `angle_power_zero_power_produces_minimum_velocity`: actual velocity at zero power = MIN_POWER_FRACTION × max_v (0.1c)
    - `angle_power_clamped_at_max`: full power clamped correctly (0.99c)
    - `angle_power_nonlinear_half_power_slower_than_linear`: non-linear curve verified through full pipeline
  - Integration test `tests/e2e_launch_arc_gradient.rs::arc_fill_fraction_increases_with_power` further validates monotonic non-linear mapping.
  - `cargo make uat` passed — 320 tests, 320 passed, 0 skipped.

## 2026-02-14 — uat-009 Verification
- **UAT**: Old bottom-center power bar UI no longer appears
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_no_power_bar.rs` with one test:
    - `no_power_bar_ui_nodes_spawned_during_launching`: enters Launching state, runs `launch_visual_system`, and asserts no new bevy_ui `Node` entities are spawned — confirming the old `PowerBarUi` entity is gone and the replacement radial arc uses Gizmos only.
  - Also verified via `grep` that `PowerBarUi` and `spawn_or_update_power_bar` no longer exist anywhere in source code (only referenced in PRD history).
  - `cargo make uat` passed — 321 tests, 321 passed, 0 skipped.

## 2026-02-14 — uat-010 Verification
- **UAT**: All existing launch unit tests pass or are updated to match new behavior
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - Ran `cargo make test` which executes the full test suite via nextest.
  - All 321 tests passed (321 passed, 0 skipped, 1 slow) in 8.2 seconds.
  - This confirms that all existing launch unit tests were updated during T-006/T-008 to match the new non-linear power curve behavior, and all new tests added throughout PRD-0014 (cancel, preview, arc, ticks, readout, no-power-bar) pass alongside the original tests.
