---
id: PRD-0009
title: "Improve Visuals: Launch UX, Menu, Outcome Screens, and Visual Polish"
status: draft
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
    uat_status: unverified
  - id: uat-002
    name: "Menu screen shows level list; clicking a level starts that level"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-003
    name: "Launch mechanic: angle line appears on click, power bar fills on drag, release launches"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-004
    name: "Trajectory trail renders behind the player, colored by total gamma"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-005
    name: "Gravity grid visualization shows field around massive objects"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-006
    name: "Success screen appears on destination collision with Next Level button"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-007
    name: "Failure screen appears on planet collision, auto-resets after short delay"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-008
    name: "Camera shake triggers on planet collision"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-009
    name: "Fade transitions between menu, game, and outcome screens"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-010
    name: "HUD shows velocity as fraction of c alongside existing clock/gamma displays"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-011
    name: "Escape key returns to menu from any game state"
    command: "cargo run (manual verification)"
    uat_status: unverified
  - id: uat-012
    name: "Completing Level 1 advances to the next level (TimeWarp)"
    command: "cargo run (manual verification)"
    uat_status: unverified
tasks:
  - id: T-001
    title: "Add GameState::Failed variant and update collision_check"
    priority: 1
    status: todo
    notes: "Replace the current Paused-on-collision with a new Failed state. collision_check sets GameState::Failed on planet hit. Remove the implicit re-launch on failure."
  - id: T-002
    title: "Implement menu screen with level selector UI"
    priority: 1
    status: todo
    notes: "Replace blank click-to-start with a bevy_ui vertical list of levels. Each entry is a Button node with text. Clicking sets CurrentLevel resource and transitions AppState::Menu -> AppState::InGame. Derive level names from CurrentLevel enum variants."
  - id: T-003
    title: "Implement two-phase launch mechanic (angle lock + power drag)"
    priority: 1
    status: todo
    notes: "Phase 1: Click sets angle (direction from player to cursor). Render a Gizmo line from player in that direction. Phase 2: Hold and drag away/toward player to set power (0-99% c). Render a power bar UI element. Release fires the player. Add a LaunchState resource (Idle, AimLocked { angle }, Launching { angle, power }) to track state machine."
  - id: T-004
    title: "Implement success screen with Next Level button"
    priority: 1
    status: todo
    notes: "On GameState::Finished, spawn a full-screen bevy_ui overlay with SUCCESS text and a Next Level button. Button click advances CurrentLevel to next variant and re-enters InGame. If no next level, return to menu."
  - id: T-005
    title: "Implement failure screen with auto-reset delay"
    priority: 1
    status: todo
    notes: "On GameState::Failed, spawn a full-screen bevy_ui overlay with FAILURE text. After ~1.5 second delay (use Timer resource), despawn overlay and reset to GameState::Paused to restart the level. Super Meat Boy style — quick flash then retry."
  - id: T-006
    title: "Add trajectory trail with gamma-based coloring"
    priority: 2
    status: todo
    notes: "Add a TrailBuffer component (Vec of (Vec2, Color)) to the player entity. Each frame during Running, push current position with a color derived from total gamma (gamma_v * gamma_g). Use Gizmos::linestrip_gradient_2d to render the trail. Cap buffer length (e.g., 2000 points). Color mapping: low gamma = blue/white, high gamma = red/orange."
  - id: T-007
    title: "Add gravitational field grid visualization"
    priority: 2
    status: todo
    notes: "Render a grid of dots/short lines showing gravitational field direction and strength. Use Gizmos to draw at grid sample points. Calculate field vector at each point by summing gravitational pull from all Mass entities. Intensity/opacity proportional to field strength. Update each frame (or every N frames for performance). Grid should cover the full screen."
  - id: T-008
    title: "Clean up HUD and add velocity display"
    priority: 2
    status: todo
    notes: "Refactor the existing clock/gamma text displays. Add current velocity as fraction of c (e.g., 0.42c). Use bevy_ui nodes instead of raw Text entities for better layout control. Group displays logically: player clock + gamma on the left, observer clock on the right, velocity indicator near the bottom or alongside clocks."
  - id: T-009
    title: "Add camera shake on collision via bevy_trauma_shake"
    priority: 3
    status: todo
    notes: "Add bevy_trauma_shake dependency. Attach Shake component to the camera entity. On planet collision (GameState::Failed transition), call shake.add_trauma(0.4). Trigger before the failure overlay spawns so the shake is visible."
  - id: T-010
    title: "Add fade transitions between screens"
    priority: 3
    status: todo
    notes: "Spawn a full-screen bevy_ui node with BackgroundColor set to black and alpha 0. On state transitions (Menu->InGame, InGame->Success/Failure, etc.), animate alpha 0->1 (fade out) then switch state then 1->0 (fade in). Use a FadeState resource and a system that interpolates alpha over time. ~0.3s per direction."
  - id: T-011
    title: "Wire up level progression (next level on success)"
    priority: 2
    status: todo
    notes: "Add a next() method to CurrentLevel that returns Option<CurrentLevel>. Success screen Next Level button calls this. If None (last level), go to menu. Ensure CurrentLevel resource is updated before re-entering InGame."
  - id: T-012
    title: "Ensure Escape returns to menu from all game states"
    priority: 2
    status: todo
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
