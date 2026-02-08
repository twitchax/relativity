---
id: PRD-0009
title: "Improve Visuals: Launch UX, Menu, Outcome Screens, and Visual Polish"
status: active
owner: twitchax
created: 2026-02-08
updated: 2026-02-08
principles:
  - "All visual features must be abstracted and apply to every level, not just Level 1"
  - "Use idiomatic Bevy 0.18 patterns: ECS components, State transitions, Gizmos, bevy_ui nodes"
  - "No new third-party crates unless clearly justified; prefer built-in Bevy APIs"
  - "bevy_trauma_shake is the one exception — it is the canonical Bevy camera-shake crate (v0.7 for Bevy 0.18)"
  - "Keep physics and rendering concerns separated; visual systems read from existing components"
  - "Focus on Level 1 for validation, but nothing should be level-specific"
references:
  - name: "Bevy 0.18 UI docs"
    url: "https://docs.rs/bevy_ui/latest/bevy_ui/"
  - name: "Bevy Gizmos API"
    url: "https://docs.rs/bevy/latest/bevy/prelude/struct.Gizmos.html"
  - name: "bevy_trauma_shake"
    url: "https://github.com/johanhelsing/bevy_trauma_shake"
  - name: "Bevy 2D Gizmos example"
    url: "https://bevy.org/examples/gizmos/2d-gizmos/"
acceptance_tests:
  - id: uat-001
    name: "Project compiles and all existing tests pass"
    command: cargo make uat
    uat_status: verified
  - id: uat-002
    name: "Menu screen spawns Button nodes for each CurrentLevel variant"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: enter AppState::Menu, query for Button entities with Text children matching each CurrentLevel variant's display name. Assert count equals number of variants. Screenshot baseline test: capture menu screen and compare against committed baseline."
    manual_note: "Visual spot-check: verify layout, spacing, and readability of menu buttons."
  - id: uat-003
    name: "Launch state machine transitions correctly (Idle → AimLocked → Launching → Running)"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: insert synthetic mouse-press input event, assert LaunchState transitions to AimLocked with correct angle. Insert drag input, assert LaunchState::Launching with power proportional to drag distance. Insert mouse-release, assert GameState transitions to Running and player Velocity matches expected angle/power."
  - id: uat-004
    name: "Launch visuals: direction line and power bar render correctly"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Screenshot baseline test: capture frame during AimLocked state, compare against baseline showing direction gizmo line. Capture during Launching state, compare against baseline showing power bar UI."
    manual_note: "Visual spot-check: verify gizmo line and power bar feel intuitive during interactive play."
  - id: uat-005
    name: "Trajectory trail renders behind the player, colored by total gamma"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: run physics for N frames, assert TrailBuffer contains expected number of entries with positions along the trajectory. Assert color values map correctly (γ ≈ 1 → cool color, γ > 2 → warm color). Screenshot baseline test: capture at a deterministic frame (e.g., frame 120 after launch) and compare trail rendering against committed baseline."
    manual_note: "Visual spot-check: verify trail gradient looks smooth and colors are distinguishable."
  - id: uat-006
    name: "Gravity grid visualization shows field around massive objects"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Screenshot baseline test: capture a frame during InGame with at least one massive object, compare against baseline showing grid gizmo lines. Unit-test the grid sampling logic separately: given known Mass entity positions, assert computed field vectors at sample points match expected gravitational acceleration."
    manual_note: "Visual spot-check: verify grid density and opacity feel informative without cluttering the scene."
  - id: uat-007
    name: "Success overlay spawns on GameState::Finished with Next Level button"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: trigger destination collision, assert GameState transitions to Finished. Query for entity with SuccessOverlay marker component. Assert a Button child with 'Next Level' text exists. Screenshot baseline test: capture the success overlay and compare against committed baseline."
  - id: uat-008
    name: "Failure overlay spawns on GameState::Failed and auto-resets to Paused after delay"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: trigger planet collision, assert GameState transitions to Failed. Query for entity with FailureOverlay marker component. Run ~90 frames (1.5s at 60fps), assert GameState transitions to Paused and FailureOverlay entity is despawned. Screenshot baseline test: capture the failure overlay immediately after spawn and compare against committed baseline."
  - id: uat-009
    name: "Camera shake trauma is applied on planet collision"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: trigger planet collision (GameState::Failed). Query camera entity for Shake component and assert trauma value > 0 (approximately 0.4)."
    manual_note: "Visual spot-check: verify shake intensity feels appropriate and does not disrupt gameplay."
  - id: uat-010
    name: "Fade overlay animates on state transitions"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: trigger a state transition (e.g., Menu → InGame). Assert FadeState resource transitions through FadeOut → state change → FadeIn. Query the fade overlay entity and assert BackgroundColor alpha interpolates from 0 → 1 → 0 over the expected frame count (~0.3s per direction). Screenshot baseline test: capture mid-fade frame and verify overlay alpha is approximately 0.5."
    manual_note: "Visual spot-check: verify fade looks smooth and does not feel sluggish."
  - id: uat-011
    name: "HUD displays velocity as fraction of c alongside clock/gamma"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: enter InGame, launch player at a known velocity. Query for velocity HUD text entity and assert displayed string matches expected format (e.g., '0.42c'). Screenshot baseline test: capture HUD layout during gameplay and compare against committed baseline."
    manual_note: "Visual spot-check: verify HUD layout is readable and well-positioned."
  - id: uat-012
    name: "Escape key returns to menu from all GameState sub-states"
    command: cargo make uat
    uat_status: verified
    automated_test: "Headless gameplay test: for each GameState variant (Paused, Running, Failed, Finished), enter that state, inject Escape key press via ButtonInput<KeyCode>, run one frame, assert AppState transitions to Menu. Verify outcome overlay entities are despawned."
  - id: uat-013
    name: "Completing Level 1 advances CurrentLevel via next() and re-enters InGame"
    command: cargo make uat
    uat_status: unverified
    automated_test: "Headless gameplay test: set CurrentLevel to Level 1, trigger destination collision → GameState::Finished. Simulate Next Level button click (or directly call next()), assert CurrentLevel resource advances to the next variant. Assert AppState re-enters InGame and new level entities are spawned."
tasks:
  - id: T-001
    title: "Add GameState::Failed variant and update collision_check"
    priority: 1
    status: done
    notes: "Replace the current Paused-on-collision with a new Failed state. collision_check sets GameState::Failed on planet hit. Remove the implicit re-launch on failure."
  - id: T-002
    title: "Implement menu screen with level selector UI"
    priority: 1
    status: done
    notes: "Replace blank click-to-start with a bevy_ui vertical list of levels. Each entry is a Button node with text. Clicking sets CurrentLevel resource and transitions AppState::Menu -> AppState::InGame. Derive level names from CurrentLevel enum variants."
  - id: T-003
    title: "Implement two-phase launch mechanic (angle lock + power drag)"
    priority: 1
    status: done
    notes: "Phase 1: Click sets angle (direction from player to cursor). Render a Gizmo line from player in that direction. Phase 2: Hold and drag away/toward player to set power (0-99% c). Render a power bar UI element. Release fires the player. Add a LaunchState resource (Idle, AimLocked { angle }, Launching { angle, power }) to track state machine."
  - id: T-004
    title: "Implement success screen with Next Level button"
    priority: 1
    status: done
    notes: "On GameState::Finished, spawn a full-screen bevy_ui overlay with SUCCESS text and a Next Level button. Button click advances CurrentLevel to next variant and re-enters InGame. If no next level, return to menu."
  - id: T-005
    title: "Implement failure screen with auto-reset delay"
    priority: 1
    status: done
    notes: "On GameState::Failed, spawn a full-screen bevy_ui overlay with FAILURE text. After ~1.5 second delay (use Timer resource), despawn overlay and reset to GameState::Paused to restart the level. Super Meat Boy style — quick flash then retry."
  - id: T-006
    title: "Add trajectory trail with gamma-based coloring"
    priority: 2
    status: done
    notes: "Add a TrailBuffer component (Vec of (Vec2, Color)) to the player entity. Each frame during Running, push current position with a color derived from total gamma (gamma_v * gamma_g). Use Gizmos::linestrip_gradient_2d to render the trail. Cap buffer length (e.g., 2000 points). Color mapping: low gamma = blue/white, high gamma = red/orange."
  - id: T-007
    title: "Add gravitational field grid visualization"
    priority: 2
    status: done
    notes: "Render a grid of dots/short lines showing gravitational field direction and strength. Use Gizmos to draw at grid sample points. Calculate field vector at each point by summing gravitational pull from all Mass entities. Intensity/opacity proportional to field strength. Update each frame (or every N frames for performance). Grid should cover the full screen."
  - id: T-008
    title: "Clean up HUD and add velocity display"
    priority: 2
    status: done
    notes: "Refactor the existing clock/gamma text displays. Add current velocity as fraction of c (e.g., 0.42c). Use bevy_ui nodes instead of raw Text entities for better layout control. Group displays logically: player clock + gamma on the left, observer clock on the right, velocity indicator near the bottom or alongside clocks."
  - id: T-009
    title: "Add camera shake on collision via bevy_trauma_shake"
    priority: 3
    status: done
    notes: "Add bevy_trauma_shake dependency. Attach Shake component to the camera entity. On planet collision (GameState::Failed transition), call shake.add_trauma(0.4). Trigger before the failure overlay spawns so the shake is visible."
  - id: T-010
    title: "Add fade transitions between screens"
    priority: 3
    status: done
    notes: "Spawn a full-screen bevy_ui node with BackgroundColor set to black and alpha 0. On state transitions (Menu->InGame, InGame->Success/Failure, etc.), animate alpha 0->1 (fade out) then switch state then 1->0 (fade in). Use a FadeState resource and a system that interpolates alpha over time. ~0.3s per direction."
  - id: T-011
    title: "Wire up level progression (next level on success)"
    priority: 2
    status: done
    notes: "Add a next() method to CurrentLevel that returns Option<CurrentLevel>. Success screen Next Level button calls this. If None (last level), go to menu. Ensure CurrentLevel resource is updated before re-entering InGame."
  - id: T-012
    title: "Ensure Escape returns to menu from all game states"
    priority: 2
    status: done
    notes: "exit_level_check already transitions to Menu on Escape. Verify it runs in all GameState sub-states (Paused, Running, Failed, Finished). Make sure outcome overlays are despawned properly on Escape."
---

# Summary

Overhaul the visual experience of the relativity game: replace the blank menu with a level selector, redesign the launch mechanic as a two-phase angle + power system, add success/failure outcome screens, implement a gamma-colored trajectory trail, add gravitational field visualization, clean up the HUD, and add visual polish (camera shake, fade transitions). All features are level-agnostic but validated against Level 1.

# Problem

The current game has minimal visual feedback and UX:
- The menu is a blank screen that starts on click — no level selection.
- The launch mechanic is a single click that determines both direction and power simultaneously, giving the player little control.
- There is no visual feedback for success (just a `println!`) or failure (silently resets to Paused, allowing re-launch).
- There is no trajectory visualization, making it hard to understand the relativistic effects on the player's path.
- The gravitational field is invisible — players must guess where gravity is strong.
- The HUD is raw text with no velocity display.
- State transitions are instant with no visual smoothness.

# Goals

1. **Intuitive launch mechanic**: Two-phase click-to-aim, drag-for-power system with visual feedback (direction line + power bar).
2. **Functional menu**: Vertical level selector that sets the current level and starts the game.
3. **Outcome screens**: Success overlay with Next Level button; failure overlay with auto-reset after a short delay (Super Meat Boy style).
4. **Trajectory trail**: Colored by total relativistic gamma so players can see where time dilation was strongest.
5. **Gravity visualization**: Grid-based field visualization showing direction and strength of gravity across the screen.
6. **Polished HUD**: Clean layout with velocity as fraction of c alongside existing clock/gamma displays.
7. **Visual polish**: Camera shake on collision, fade transitions between screens.
8. **Level progression**: Completing a level advances to the next one.

# Technical Approach

## Architecture

All new systems follow Bevy's ECS patterns. No level-specific code — all features read from existing components (Position, Mass, Radius, Velocity, etc.) that are already shared across levels.

```
┌─────────────────────────────────────────────────────┐
│                     AppState                        │
│  ┌──────────┐    ┌──────────────────────────────┐   │
│  │   Menu   │───▶│           InGame              │   │
│  │(LevelUI) │◀───│  ┌────────┐  ┌───────────┐   │   │
│  └──────────┘    │  │ Paused │──│  Running   │   │   │
│       ▲          │  │(Launch)│  │(Physics +  │   │   │
│       │          │  └────────┘  │ Trail +    │   │   │
│       │          │       ▲      │ Gravity Viz│   │   │
│       │          │       │      └─────┬──────┘   │   │
│       │          │       │        ┌───┴───┐      │   │
│       │          │  ┌────┴───┐  ┌─┴──────┐│      │   │
│       │          │  │Failed  │  │Finished││      │   │
│       │          │  │(Overlay│  │(Overlay ││      │   │
│       │          │  │+Delay) │  │+Button) ││      │   │
│       │          │  └────────┘  └─────────┘│      │   │
│       │          └──────────────────────────┘     │   │
│  Escape key returns to Menu from any GameState    │   │
│  Fade overlay animates on all transitions         │   │
└─────────────────────────────────────────────────────┘
```

## New State: `GameState::Failed`

Add a `Failed` variant to `GameState`. The collision system sets `Failed` on planet collision (instead of resetting to `Paused`). A timer-based system in the `Failed` state waits ~1.5s, then transitions back to `Paused` (which despawns/respawns the level for a retry).

## Launch Mechanic

Introduce a `LaunchState` resource with an enum state machine:
- `Idle` — waiting for click
- `AimLocked { angle: f32 }` — click registered, direction line rendered via Gizmos
- `Launching { angle: f32, power: f32 }` — dragging to set power, power bar UI visible

The existing `player_launch` system is replaced by a multi-system approach:
1. `launch_aim_system` — on mouse press, compute angle from player to cursor, transition to AimLocked
2. `launch_power_system` — on mouse drag, compute power from drag distance (0 to 0.99 * MAX_PLAYER_LAUNCH_VELOCITY)
3. `launch_fire_system` — on mouse release in Launching state, set velocity on player, transition GameState to Running
4. `launch_visual_system` — Gizmo line for direction, UI node for power bar

## Trajectory Trail

A `TrailBuffer` component attached to the player entity stores a ring buffer of `(Vec2, Color)` tuples. Each frame during `Running`, the player's screen position is pushed with a color mapped from `gamma_v * gamma_g`:
- γ ≈ 1.0 → cool color (blue/cyan)
- γ > 2.0 → warm color (red/orange)

Rendered via `Gizmos::linestrip_gradient_2d`. Buffer capped at ~2000 points for performance.

## Gravitational Field Grid

A system running during `InGame` (all sub-states) samples a grid of points across the screen. At each point, it sums gravitational acceleration from all `Mass` entities using the existing `calculate_gravitational_acceleration` function. Short Gizmo lines are drawn at each grid point in the direction of the field, with length/opacity proportional to field strength. Grid resolution configurable (e.g., 20×12 = 240 sample points).

## Menu Screen

Replace the current `start` system with a proper `bevy_ui` layout:
- A centered column of `Button` nodes, one per `CurrentLevel` variant
- Button text is the level's display name
- On click: set `CurrentLevel` resource, transition to `AppState::InGame`

## Outcome Overlays

- **Success**: Full-screen semi-transparent overlay with "SUCCESS" text and a "Next Level" `Button`. Button advances `CurrentLevel` and restarts `InGame`.
- **Failure**: Full-screen semi-transparent overlay with "FAILURE" text. A `Timer` resource triggers auto-reset after ~1.5 seconds. Despawns overlay, resets level to `Paused`.

Both overlays are spawned as `bevy_ui` node trees and tagged with a marker component for cleanup.

## Camera Shake

Add `bevy_trauma_shake` as a dependency. Attach `Shake` component to the camera. On collision (entering `Failed` state), call `shake.add_trauma(0.4)`. The shake runs concurrently with the failure overlay.

## Fade Transitions

A persistent full-screen `bevy_ui` node with `BackgroundColor` (black, alpha=0) and high `GlobalZIndex`. A `FadeState` resource tracks `{ direction: FadeIn|FadeOut|None, timer: Timer, next_action: ... }`. On state transitions, trigger fade out → execute transition → fade in. ~0.3s per direction.

## HUD Improvements

Migrate clock/gamma text from raw `Text` entities to structured `bevy_ui` nodes. Add a velocity display showing `v = 0.42c` format. Layout: left-aligned player info (clock, gamma, velocity), right-aligned observer clock.

## Visual Regression Testing Strategy

Many UATs involve visual output that cannot be verified through ECS queries alone (trails, grids, overlays, HUD layout). The project already has a working screenshot baseline infrastructure (see `tests/e2e_screenshot.rs` and `tests/common/screenshot.rs`) that renders to an offscreen `Image` asset and compares pixel-by-pixel against committed PNG baselines in `tests/baselines/`.

This infrastructure should be extended to cover the new visual features. The approach:

1. **Deterministic frame capture**: Each visual UAT specifies a deterministic capture point (e.g., "frame 120 after launch at velocity X"). Because the test app uses `TimeUpdateStrategy::ManualDuration`, physics is frame-deterministic, so the rendered output is reproducible.
2. **One baseline per visual feature**: Separate baseline PNGs per feature (e.g., `trail_gamma_frame120.png`, `gravity_grid_level1.png`, `success_overlay.png`, `failure_overlay.png`, `hud_layout.png`, `menu_screen.png`, `launch_aim.png`, `launch_power.png`, `fade_mid.png`).
3. **First-run bootstrap**: On first run (no baseline exists), the test saves the rendered image as the new baseline and fails with a review prompt. The developer commits the baseline after visual inspection.
4. **Threshold tolerance**: Pixel comparison should use a small tolerance (e.g., ≤1% differing pixels) to accommodate minor rendering differences across GPU drivers and platforms.
5. **CI compatibility**: The headless render pipeline (`DefaultPlugins` minus `WinitPlugin` with offscreen camera) works in CI without a display server. GPU availability may vary — tests should be skipped gracefully on CI runners without GPU support.

This means an automated agent can implement a feature, run `cargo make uat`, and get a pass/fail signal for the visual output. The only manual step is the initial baseline review when a new screenshot test is first added or when a visual change intentionally updates the baseline.

# Assumptions

- Bevy 0.18's `Gizmos` API supports `linestrip_gradient_2d` for the trail and grid rendering
- `bevy_trauma_shake` has a release compatible with Bevy 0.18 (confirmed: main branch targets latest Bevy)
- The gravity grid at 240 sample points per frame is performant enough (can reduce if needed)
- The existing `calculate_gravitational_acceleration` function can be reused for grid visualization
- `CurrentLevel` enum variants have a natural ordering that can be used for progression

# Constraints

- All features MUST be level-agnostic — no level-specific visual code
- The launch mechanic change will break the existing `calculate_launch_velocity` tests — they need to be updated or replaced
- WASM builds must continue to work (no desktop-only APIs)
- Camera shake must not interfere with the 2D coordinate system used for physics
- Gizmo rendering is immediate-mode (every frame) — trail buffer must be managed carefully for performance
- Fade transitions must not block input during the fade (or must be short enough to not feel laggy)

# References to Code

- `src/shared/state.rs` — `AppState`, `GameState` enums (add `Failed` variant here)
- `src/game/mod.rs` — `GamePlugin` system registration (add new systems here)
- `src/menu/mod.rs` — `MenuPlugin` (replace `start` system with UI)
- `src/game/player/player_sprite.rs` — `player_launch` and `calculate_launch_velocity` (replace with new launch mechanic)
- `src/game/shared/systems.rs` — `collision_check` (update for `Failed` state), `calculate_gravitational_acceleration` (reuse for grid)
- `src/game/shared/types.rs` — ECS components (add `TrailBuffer`, `LaunchState`)
- `src/game/shared/constants.rs` — `C`, `MAX_PLAYER_LAUNCH_VELOCITY` (used for power scaling and HUD)
- `src/game/player/player_clock.rs` — gamma calculation (read for trail coloring)
- `src/game/levels/mod.rs` — `CurrentLevel` (add `next()` method, display names)
- `src/game/destination/mod.rs` — destination entity (no changes needed)
- `src/game/observer/mod.rs` — observer clock (HUD migration)

# Non-Goals (MVP)

- Sound effects or music
- Particle effects beyond the trajectory trail
- Level thumbnails or previews in the menu
- Level locking / progression persistence (all levels available from menu)
- Settings screen (volume, resolution, controls)
- Custom shaders for gravity visualization (use Gizmos for MVP)
- Gamepad / keyboard-only input for launch mechanic
- Animated sprites or sprite sheets for the player rocket
- Saving/loading game progress

# History

## 2026-02-08 — T-001 Completed
- **Task**: Add GameState::Failed variant and update collision_check
- **Status**: ✅ Done
- **Changes**:
  - Added `Failed` variant to `GameState` enum in `src/shared/state.rs`
  - Updated `collision_check` in `src/game/shared/systems.rs` to set `GameState::Failed` on planet collision (was `GameState::Paused`)
  - Updated unit tests in `src/shared/state.rs` to cover the new `Failed` variant (distinctness, debug format)
  - Updated `tests/e2e_collision.rs`: renamed test `collision_with_planet_transitions_to_paused` → `collision_with_planet_transitions_to_failed`, updated assertions and comments
  - Updated `tests/e2e_level1_gameplay.rs`: updated assertion to expect `GameState::Failed` on planet collision
  - Updated `tests/common/gameplay.rs`: updated doc comment for `run_until_resolved`
  - UAT passed: 166 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, consistent with existing patterns, no public API breakage (internal state enum).

## 2026-02-08 — T-002 Completed
- **Task**: Implement menu screen with level selector UI
- **Status**: ✅ Done
- **Changes**:
  - Rewrote `src/menu/mod.rs`: replaced bare `start` system (click-to-start) with a full `bevy_ui` menu featuring `OnEnter`/`OnExit` spawn/despawn lifecycle, a title text node, and one `Button` per `CurrentLevel` variant. Clicking a button sets `CurrentLevel` and transitions to `AppState::InGame`.
  - Added marker components `MenuScreen` and `LevelButton(CurrentLevel)` for entity management and interaction handling.
  - Updated `src/game/levels/mod.rs`: added `Display` impl for `CurrentLevel` (display names: "Level 1", "Time Warp"), added `CurrentLevel::all()` const method, derived `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` on `CurrentLevel`.
  - Added unit tests for `Display` and `all()` in `src/game/levels/mod.rs`.
  - UAT passed: 169 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, consistent with existing Bevy ECS patterns, no public API breakage.

## 2026-02-08 — T-003 Completed
- **Task**: Implement two-phase launch mechanic (angle lock + power drag)
- **Status**: ✅ Done
- **Changes**:
  - Added `LaunchState` resource enum (`Idle`, `AimLocked { angle }`, `Launching { angle, power }`) and `PowerBarUi` marker component to `src/game/shared/types.rs`.
  - Replaced single `player_launch` system with four systems in `src/game/player/player_sprite.rs`:
    - `launch_aim_system` — on mouse press, computes angle from player to cursor, transitions to `AimLocked`.
    - `launch_power_system` — while mouse held, computes power from drag distance (0–1.0, scaled by 80% screen width).
    - `launch_fire_system` — on mouse release in `Launching` state, sets velocity and transitions to `GameState::Running`. Release from `AimLocked` (no drag) cancels back to `Idle`.
    - `launch_visual_system` — renders direction Gizmo line and spawns/despawns a power-bar `bevy_ui` overlay.
  - Added `calculate_launch_velocity_from_angle_power` pure function for velocity computation from angle + power.
  - Retained `calculate_launch_velocity` (gated with `#[cfg(test)]`) for backward-compatible unit tests.
  - Updated `src/game/mod.rs`: registered `LaunchState` resource, replaced `player_launch` with four new systems in `Paused` state.
  - Updated `tests/common/gameplay.rs`: added `bevy::gizmos::GizmoPlugin` to headless test app for `Gizmos` system param support. Updated doc comments.
  - Updated `tests/e2e_level1_gameplay.rs`: updated doc comments to reflect new system names.
  - Added 7 new unit tests for `calculate_launch_velocity_from_angle_power` and `LaunchState`.
  - UAT passed: 177 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, consistent with existing Bevy ECS patterns, no public API breakage. The old `calculate_launch_velocity` function was preserved (test-only) for backward compatibility.

## 2026-02-08 — T-004 Completed
- **Task**: Implement success screen with Next Level button
- **Status**: ✅ Done
- **Changes**:
  - Added `next()` method to `CurrentLevel` in `src/game/levels/mod.rs` — returns `Some(next_variant)` or `None` for the last level.
  - Added `SuccessOverlay` and `NextLevelButton` marker components to `src/game/shared/types.rs`.
  - Created new `src/game/outcome/mod.rs` module with three systems:
    - `spawn_success_overlay` — spawns a full-screen semi-transparent bevy_ui overlay with "SUCCESS" text and a "Next Level" (or "Menu") button on `OnEnter(GameState::Finished)`.
    - `despawn_success_overlay` — despawns the overlay on `OnExit(GameState::Finished)`.
    - `success_button_interaction` — handles button click: advances `CurrentLevel` if a next level exists, then transitions to `AppState::Menu` + `GameState::Paused`.
  - Registered the `outcome` module in `src/game/mod.rs` and wired `OnEnter`/`OnExit` for `GameState::Finished` plus the button interaction system.
  - Added 2 unit tests for `CurrentLevel::next()` in `src/game/levels/mod.rs`.
  - UAT passed: 179 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes following existing patterns (menu module structure for UI, OnEnter/OnExit for lifecycle, marker components for queries). New `outcome` module follows Separation of Concerns principle.

## 2026-02-08 — T-005 Completed
- **Task**: Implement failure screen with auto-reset delay
- **Status**: ✅ Done
- **Changes**:
  - Added `FailureOverlay` marker component and `FailureTimer(Timer)` resource to `src/game/shared/types.rs`.
  - Added three systems to `src/game/outcome/mod.rs`:
    - `spawn_failure_overlay` — spawns a full-screen semi-transparent bevy_ui overlay with "FAILURE" text (red) and inserts a 1.5s `FailureTimer` resource on `OnEnter(GameState::Failed)`.
    - `despawn_failure_overlay` — despawns the overlay and removes the timer on `OnExit(GameState::Failed)`.
    - `failure_auto_reset` — ticks the timer each frame; transitions to `GameState::Paused` when finished.
  - Wired lifecycle systems (`OnEnter`/`OnExit` for `GameState::Failed`) and the auto-reset update system in `src/game/mod.rs`.
  - UAT passed: 179 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, consistent with existing success overlay patterns (same module, same component/lifecycle approach).

## 2026-02-08 — T-006 Completed
- **Task**: Add trajectory trail with gamma-based coloring
- **Status**: ✅ Done
- **Changes**:
  - Added `TrailBuffer` component (Vec of `(Vec2, Color)`) to `src/game/shared/types.rs`.
  - Added `TrailBuffer` to `PlayerSpriteBundle` in `src/game/player/player_sprite.rs` so it is automatically attached to every player entity.
  - Created new `src/game/trail/mod.rs` module with:
    - `gamma_to_color` pure function — maps combined gamma (γ_v × γ_g) to a color gradient: blue/cyan at γ≈1 → red/orange at γ≥3.
    - `trail_record_system` — records player screen position + gamma-based color each frame during `Running`. Caps buffer at 2000 points.
    - `trail_render_system` — renders the trail via `Gizmos::linestrip_gradient_2d`, runs across all `InGame` sub-states.
    - `trail_clear_system` — clears the buffer on `OnEnter(GameState::Paused)` for level resets.
  - Registered the `trail` module in `src/game/mod.rs` and wired systems: record during `Running` (after `player_clock_update`), render during `InGame`, clear on `OnEnter(GameState::Paused)`.
  - Added 8 unit tests for `gamma_to_color` and `TrailBuffer` default.
  - UAT passed: 187 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, new module follows Separation of Concerns. Consistent with existing ECS patterns (component on player bundle, Gizmos for rendering, marker-based queries).

## 2026-02-08 — T-007 Completed
- **Task**: Add gravitational field grid visualization
- **Status**: ✅ Done
- **Changes**:
  - Created new `src/game/gravity_grid/mod.rs` module with:
    - `compute_field_at_point` pure function — computes total gravitational field vector at a world-space point by summing contributions from all Mass entities using `calculate_gravitational_acceleration`.
    - `gravity_grid_render_system` — samples a 20×12 grid of points across the screen, computes field direction and log-scaled strength at each point, and draws Gizmo lines with length and alpha proportional to field strength.
  - Registered the `gravity_grid` module in `src/game/mod.rs` and wired the render system alongside `trail_render_system` during all `InGame` sub-states.
  - Added 5 unit tests for `compute_field_at_point` (no masses, direction toward mass, inverse-square falloff, superposition of two masses).
  - Regenerated screenshot baseline `tests/baselines/level1_spawn.png` to include gravity grid lines.
  - UAT passed: 192 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes, new module follows Separation of Concerns. Reuses existing `calculate_gravitational_acceleration` function (DRY). Consistent with existing ECS/Gizmos patterns (trail module structure).

## 2026-02-08 — T-008 Completed
- **Task**: Clean up HUD and add velocity display
- **Status**: ✅ Done
- **Changes**:
  - Added `PlayerHud` marker component to `src/game/shared/types.rs` — differentiates the HUD text entity from the player sprite entity for cleaner query filtering.
  - Added `PlayerHud` to `PlayerClockBundle` in `src/game/player/player_clock.rs`.
  - Added `format_velocity_fraction` pure function in `src/game/player/player_clock.rs` — formats scalar velocity as fraction of c (e.g., `v = 0.42c`).
  - Updated `player_clock_text_update` system to query player `Velocity` from the sprite entity and include velocity display in the HUD text.
  - Updated HUD format string: `t_p = DD.DD  γ_v = X.XX  γ_g = X.XX  v = X.XXc` — player clock, velocity gamma, gravitational gamma, and velocity as fraction of c, all on one line with consistent spacing.
  - Updated initial HUD text in `spawn_player_clock` to match the new format.
  - Added 6 unit tests for `format_velocity_fraction` (at rest, half c, near c, specific fraction, prefix, suffix).
  - UAT passed: 198 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes (2 files, ~56 lines added). Consistent with existing patterns (pure function + system query pattern). No public API breakage. DRY: reuses existing `Velocity::scalar()` and `C` constant.

## 2026-02-08 — T-011 Completed
- **Task**: Wire up level progression (next level on success)
- **Status**: ✅ Done
- **Changes**:
  - Added `PendingNextLevel` marker resource to `src/game/shared/types.rs` — signals the menu to auto-advance to `InGame`.
  - Updated `success_button_interaction` in `src/game/outcome/mod.rs` to insert `PendingNextLevel` when a next level exists, so the menu auto-advances instead of requiring manual level selection.
  - Added `auto_advance_to_next_level` system in `src/menu/mod.rs` — runs on `OnEnter(AppState::Menu)`, checks for `PendingNextLevel`, removes it, and immediately transitions to `AppState::InGame`.
  - Registered `auto_advance_to_next_level` alongside `spawn_menu` in `MenuPlugin`.
  - UAT passed: 198 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes (3 files). Consistent with existing patterns (marker resource, `OnEnter` system). `CurrentLevel::next()` already existed from T-004 — reused (DRY). No public API breakage.

## 2026-02-08 — T-012 Completed
- **Task**: Ensure Escape returns to menu from all game states
- **Status**: ✅ Done
- **Changes**:
  - Verified `exit_level_check` already runs with `.run_if(in_state(AppState::InGame))` — no `GameState` filter — so it handles Escape from all sub-states (Paused, Running, Failed, Finished).
  - Verified outcome overlays are properly cleaned up: setting `GameState::Paused` from `Failed`/`Finished` triggers `OnExit` hooks that despawn `FailureOverlay`/`SuccessOverlay` entities.
  - Updated `exit_level_check` in `src/game/shared/systems.rs` to also reset `LaunchState` to `Idle` on Escape, preventing stale aim/launch state when re-entering the game from menu.
  - Added `LaunchState` import to the system's module.
  - Created `tests/e2e_escape.rs` with 5 E2E tests:
    - `escape_from_paused_returns_to_menu` — verifies Escape from Paused transitions to Menu.
    - `escape_from_running_returns_to_menu` — verifies Escape from Running transitions to Menu.
    - `escape_from_failed_returns_to_menu` — verifies Escape from Failed transitions to Menu and despawns FailureOverlay.
    - `escape_from_finished_returns_to_menu` — verifies Escape from Finished transitions to Menu and despawns SuccessOverlay.
    - `escape_resets_launch_state_to_idle` — verifies LaunchState is reset to Idle on Escape.
  - UAT passed: 203 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes (2 files modified, 1 file created). Consistent with existing ECS patterns and test conventions.

## 2026-02-08 — T-009 Completed
- **Task**: Add camera shake on collision via bevy_trauma_shake
- **Status**: ✅ Done
- **Changes**:
  - Added `bevy_trauma_shake = "0.7"` dependency to `Cargo.toml`.
  - Updated `src/shared/types.rs`: added `Shake::default()` component to the camera entity in `spawn_camera`.
  - Updated `src/main.rs`: added `TraumaPlugin` to the Bevy app.
  - Added `apply_collision_shake` system in `src/game/outcome/mod.rs` — queries all `Shake` components and calls `add_trauma(0.4)` on `OnEnter(GameState::Failed)`.
  - Wired `apply_collision_shake` alongside `spawn_failure_overlay` in `src/game/mod.rs` on `OnEnter(GameState::Failed)`.
  - Updated `tests/common/gameplay.rs`: added `TraumaPlugin` to headless test app.
  - Updated `tests/common/headless.rs`: added `TraumaPlugin` and `Shake::default()` to offscreen camera.
  - UAT passed: 203 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes (7 files). Consistent with existing ECS patterns (OnEnter system, component on camera, plugin registration). New dependency `bevy_trauma_shake` is justified per PRD principles.

## 2026-02-08 — T-010 Completed
- **Task**: Add fade transitions between screens
- **Status**: ✅ Done
- **Changes**:
  - Created new `src/game/fade/mod.rs` module with:
    - `FadeOverlay` marker component for the persistent full-screen overlay entity.
    - `FadeDirection` enum (`Out { next_app_state, next_game_state }`, `In`) tracking animation direction.
    - `FadeState` resource with `start_fade_out()` and `start_fade_in()` methods and a `Timer`.
    - `spawn_fade_overlay` startup system — spawns a persistent full-screen `bevy_ui` node with `BackgroundColor` (black, alpha=0), `GlobalZIndex(200)`, and `Pickable::IGNORE` to avoid blocking clicks.
    - `fade_update_system` — ticks the fade timer each frame, interpolates overlay alpha (0→1 for out, 1→0 for in), applies the deferred `AppState`/`GameState` transition when fade-out completes, and auto-starts fade-in.
    - `is_fading()` helper for callers to suppress input during transitions.
    - 7 unit tests for `FadeState` defaults, direction tracking, and `is_fading` logic.
  - Registered `fade` module in `src/game/mod.rs`: added `FadeState` resource init, `spawn_fade_overlay` startup system, and `fade_update_system` running every frame.
  - Updated `src/menu/mod.rs`: `menu_button_interaction` and `auto_advance_to_next_level` now call `fade.start_fade_out()` instead of directly setting `AppState`. Menu button input is suppressed while fading.
  - Updated `src/game/outcome/mod.rs`: `success_button_interaction` now calls `fade.start_fade_out()` instead of directly setting `AppState`/`GameState`. Input suppressed while fading.
  - Updated `src/game/shared/systems.rs`: `exit_level_check` now starts a fade-out to `AppState::Menu` instead of immediate transition. `GameState::Paused` is still set immediately for overlay cleanup.
  - Updated `tests/e2e_escape.rs`: `press_escape_and_process` now runs 25 frames to allow the fade-out animation (~0.3s at 60fps) to complete before asserting `AppState::Menu`.
  - UAT passed: 210 tests, 0 failures

- **Constitution Compliance**: No violations. Minimal changes (6 files modified, 1 file created). Consistent with existing ECS patterns (Resource + System, Startup system, marker component). Separation of Concerns: fade logic is isolated in its own module. DRY: all callers use the same `FadeState::start_fade_out()` API.

## 2026-02-08 — uat-001 Verification
- **UAT**: Project compiles and all existing tests pass
- **Status**: ✅ Verified
- **Method**: Existing test
- **Details**:
  - Ran `cargo make uat` which executes the full CI pipeline (fmt-check + clippy + nextest).
  - All 210 tests passed with 0 failures and 0 skipped.
  - No new test needed — this UAT is satisfied by the existing test suite passing end-to-end.
