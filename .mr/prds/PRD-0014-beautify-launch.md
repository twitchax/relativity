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
    uat_status: unverified
  - id: uat-002
    name: "Click locks aim direction and transitions LaunchState to AimLocked"
    command: cargo make uat
    uat_status: unverified
  - id: uat-003
    name: "Drag along aim direction fills radial arc around player with correct color gradient"
    command: cargo make uat
    uat_status: unverified
  - id: uat-004
    name: "Arc tick marks appear at 0.25c, 0.5c, 0.75c, and 0.9c"
    command: cargo make uat
    uat_status: unverified
  - id: uat-005
    name: "Numeric velocity readout (e.g. 0.45c) displays near the arc during Launching phase"
    command: cargo make uat
    uat_status: unverified
  - id: uat-006
    name: "Release fires player at correct angle and power; GameState transitions to Running"
    command: cargo make uat
    uat_status: unverified
  - id: uat-007
    name: "Right-click or Escape cancels launch at any phase and returns to Idle"
    command: cargo make uat
    uat_status: unverified
  - id: uat-008
    name: "Power maps to 0.1c–0.99c range with non-linear (exponential ease-in) curve"
    command: cargo make uat
    uat_status: unverified
  - id: uat-009
    name: "Old bottom-center power bar UI no longer appears"
    command: cargo make uat
    uat_status: unverified
  - id: uat-010
    name: "All existing launch unit tests pass or are updated to match new behavior"
    command: cargo make test
    uat_status: unverified
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
    status: todo
    notes: "Short radial lines at fixed angular positions corresponding to those velocities. Rendered via Gizmos during Launching phase."
  - id: T-005
    title: "Add numeric velocity readout near the arc"
    priority: 2
    status: todo
    notes: "Spawn a small text entity (or use Gizmos text if available) near the arc tip showing e.g. '0.45c'. Update each frame during Launching. Despawn on Idle."
  - id: T-006
    title: "Implement non-linear power curve (exponential ease-in, 0.1c–0.99c)"
    priority: 2
    status: todo
    notes: "Replace linear power mapping. Raw power 0.0–1.0 from drag → apply exponential ease-in → map to 0.1c–0.99c. Update `calculate_launch_velocity_from_angle_power` or add a new mapping function. Minimum launch velocity is 0.1c."
  - id: T-007
    title: "Update launch_visual_system to use new arc + dotted line visuals"
    priority: 1
    status: done
    notes: "Refactor `launch_visual_system` to render: AimLocked → solid direction line + empty arc outline; Launching → solid direction line (length scaled by power) + filled arc + tick marks + readout."
  - id: T-008
    title: "Update and add unit tests for new power curve and arc rendering logic"
    priority: 3
    status: todo
    notes: "Update existing tests for `calculate_launch_velocity_from_angle_power`. Add tests for non-linear mapping, 0.1c minimum, cancel behavior."
  - id: T-009
    title: "Clean up dead code (PowerBarUi, spawn_or_update_power_bar)"
    priority: 3
    status: todo
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
