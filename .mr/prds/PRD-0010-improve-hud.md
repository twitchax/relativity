---
id: PRD-0010
title: "Improve HUD: Bottom-Anchored Chrome with bevy_lunex"
status: active
owner: twitchax
created: 2026-02-08
updated: 2026-02-08

principles:
  - Use bevy_lunex for HUD layout and styling; keep existing bevy_ui for menu/outcome screens unchanged
  - Retain all existing HUD data (t_p, γ_v, γ_g, v, t_o) with identical semantics
  - Dark sci-fi aesthetic consistent with the game's existing visual language
  - WASM-compatible (bevy_lunex supports wasm feature flag)
  - Minimal changes to game logic systems; only spawn and text-update code should change

references:
  - name: "bevy_lunex crate (v0.6, Bevy 0.18)"
    url: "https://crates.io/crates/bevy_lunex"
  - name: "bevy_lunex documentation"
    url: "https://docs.rs/bevy_lunex/0.6.0/bevy_lunex/"
  - name: "bevy_lunex book"
    url: "https://bytestring-net.github.io/bevy_lunex/"
  - name: "Bevypunk example (production-ready bevy_lunex showcase)"
    url: "https://github.com/IDEDARY/Bevypunk"
  - name: "bytestring-net/bevy_lunex GitHub (864 stars)"
    url: "https://github.com/bytestring-net/bevy_lunex"

acceptance_tests:
  - id: uat-001
    name: "HUD renders at bottom of screen with chrome panels"
    command: cargo make uat
    uat_status: verified
  - id: uat-002
    name: "Player panel shows t_p, γ_v, γ_g, v with correct values"
    command: cargo make uat
    uat_status: unverified
  - id: uat-003
    name: "Observer panel shows t_o with correct values"
    command: cargo make uat
    uat_status: unverified
  - id: uat-004
    name: "HUD adapts to different window sizes and aspect ratios"
    command: cargo make uat
    uat_status: unverified
  - id: uat-005
    name: "WASM build compiles and runs with new HUD"
    command: cargo make build-web
    uat_status: unverified
  - id: uat-006
    name: "CI pipeline passes (fmt-check, clippy, nextest)"
    command: cargo make ci
    uat_status: unverified

tasks:
  - id: T-001
    title: "Add bevy_lunex dependency to Cargo.toml"
    priority: 1
    status: done
    notes: "Add bevy_lunex = \"0.6\" to [dependencies]. Add wasm feature for WASM target in [target.'cfg(target_arch = \"wasm32\")'.dependencies]."
  - id: T-002
    title: "Create HUD module with bevy_lunex layout root and plugin"
    priority: 2
    status: done
    notes: "Create src/game/hud/ module. Add a HudPlugin that registers UiLunexPlugins and spawns the HUD layout root (UiLayoutRoot::new_2d + UiFetchFromCamera). Register in game plugin."
  - id: T-003
    title: "Build bottom-anchored HUD bar with two segmented panels"
    priority: 3
    status: done
    notes: "Use UiLayout::boundary() or window() to position a bar at the bottom of the screen. Create left panel (player stats) and right panel (observer clock). Use Sprite with 9-slice for chrome background. Semi-transparent dark background (0.1, 0.1, 0.15, 0.85) with thin cyan/white border accent."
  - id: T-004
    title: "Create or source HUD chrome sprite assets"
    priority: 3
    status: done
    notes: "Create minimal panel background sprites for 9-slice rendering (dark panel with border glow). Place in assets/sprites/hud/. Can be simple programmatic textures or hand-crafted PNGs."
  - id: T-005
    title: "Wire player stats into left HUD panel"
    priority: 4
    status: done
    notes: "Display t_p, γ_v, γ_g, v as separate labeled readouts using Text2d + UiTextSize. Each on its own line or in a grid within the panel. Update player_clock_text_update to target new entities."
  - id: T-006
    title: "Wire observer clock into right HUD panel"
    priority: 4
    status: done
    notes: "Display t_o in the right panel. Update observer_clock_text_update to target new entity."
  - id: T-007
    title: "Remove old top-positioned HUD spawn code"
    priority: 5
    status: done
    notes: "Remove spawn_player_clock and spawn_observer_clock old Node-based spawning. Keep pure functions (calculate_*, format_*) untouched."
  - id: T-008
    title: "Mark HUD entities with GameItem for lifecycle management"
    priority: 5
    status: done
    notes: "Ensure all new HUD entities include the GameItem component so they are properly despawned on state transitions."
  - id: T-009
    title: "Verify WASM build and test cross-platform"
    priority: 6
    status: done
    notes: "Run cargo make build-web. Ensure bevy_lunex wasm feature is enabled for wasm32 target. Test in browser."
  - id: T-010
    title: "Run full CI and fix any clippy/fmt issues"
    priority: 7
    status: done
    notes: "Run cargo make ci. Fix any pedantic clippy lints from new code. Ensure all existing tests pass."
---

# Summary

Relocate the game HUD from its current position (bare text at top-left and top-right) to a polished, chrome-styled bottom bar using **bevy_lunex** as the layout engine. The HUD will feature two segmented panels — a left panel for player relativistic stats (`t_p`, `γ_v`, `γ_g`, `v`) and a right panel for the observer clock (`t_o`) — with a dark sci-fi aesthetic that matches the game's existing visual language.

# Problem

The current HUD is plain unstyled text floating at the top corners of the screen. It has no background, no borders, and no visual chrome. This looks unfinished and fails to match the quality of other visual elements (menu buttons, outcome screens, trajectory trails) that were polished in PRD-0009. Additionally, the top positioning competes for visual space with the game action area.

# Goals

1. Move the HUD to the bottom of the screen, freeing the top for unobstructed gameplay view.
2. Add polished visual chrome (semi-transparent dark panels with glowing border accents) using bevy_lunex's layout and sprite-based rendering.
3. Separate player stats and observer clock into distinct, clearly labeled panels.
4. Display each readout as an individual labeled element rather than a single concatenated text string.
5. Ensure the HUD scales correctly across all window sizes and aspect ratios.
6. Maintain WASM compatibility for the GitHub Pages deployment.

# Technical Approach

## Library Choice: bevy_lunex v0.6

**bevy_lunex** (864 GitHub stars, actively maintained) is a retained layout engine for Bevy ECS. It is chosen over native `bevy_ui` for the following reasons:

- **Sprite-based rendering**: Allows 9-slice sprite panels for high-quality chrome that would be difficult with bevy_ui's box model alone.
- **ECS-native**: Uses standard Bevy `Transform`, `Sprite`, `Text2d` components — no paradigm shift from the existing codebase.
- **Aspect-ratio aware**: Built-in support for responsive layouts that handle any window size without deformation.
- **WASM support**: Has a `wasm` feature flag and a live WASM demo (Bevypunk on itch.io).
- **Bevy 0.18 compatible**: v0.6 explicitly targets Bevy 0.18.

The library will only be used for the in-game HUD. The menu system and outcome screens will remain on native `bevy_ui` to minimize change scope.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     Game Viewport                            │
│                                                              │
│                    (gameplay area)                            │
│                                                              │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  ┌─── Player Panel ───────────┐  ┌─── Observer Panel ────┐  │
│  │ t_p = 12.34    γ_v = 1.02  │  │ t_o = 14.56           │  │
│  │ γ_g = 1.01     v = 0.42c   │  │                       │  │
│  └────────────────────────────┘  └───────────────────────┘  │
│  ░░░░░░░░░░░░░ Bottom HUD Bar ░░░░░░░░░░░░░░░░░░░░░░░░░░░  │
└──────────────────────────────────────────────────────────────┘
```

## Component Hierarchy

```
UiLayoutRoot (2D, camera-synced)
└── HUD Bar (boundary: bottom 12% of screen)
    ├── Player Panel (left 60%, 9-slice sprite background)
    │   ├── Label: "t_p = {value}" (Text2d)
    │   ├── Label: "γ_v = {value}" (Text2d)
    │   ├── Label: "γ_g = {value}" (Text2d)
    │   └── Label: "v = {value}c"  (Text2d)
    └── Observer Panel (right 35%, 9-slice sprite background)
        └── Label: "t_o = {value}" (Text2d)
```

## Styling

- **Panel background**: Semi-transparent dark (`rgba(0.1, 0.1, 0.15, 0.85)`) with 9-slice sprite for border detail.
- **Border accent**: Thin cyan glow (`rgba(0.3, 0.8, 1.0, 0.7)`) baked into the panel sprite edges.
- **Text**: HackNerdFontMono-Regular.ttf (existing font), sized proportionally to panel height via `UiTextSize`.
- **Text color**: Soft white (`rgba(0.9, 0.95, 1.0, 1.0)`) for readability against dark background.

## System Changes

- `spawn_player_clock` and `spawn_observer_clock` will be replaced by a unified `spawn_hud` function that builds the bevy_lunex tree.
- `player_clock_text_update` will be updated to query individual `Text2d` components (one per stat) instead of a single `Text` node.
- `observer_clock_text_update` will be similarly updated.
- Pure calculation functions (`calculate_velocity_gamma`, `calculate_gravitational_gamma`, etc.) remain completely untouched.
- `player_clock_update` (the ECS system that updates Clock/Gamma components) remains untouched.

# Assumptions

- bevy_lunex v0.6 is stable enough for production use with Bevy 0.18 (confirmed by crate metadata and active maintenance).
- bevy_lunex's `UiLayoutRoot::new_2d()` with `UiFetchFromCamera` works correctly in the project's 2D camera setup.
- The 9-slice sprite approach for panel chrome will render correctly in both native and WASM builds.
- The existing `GameItem` cleanup system will correctly despawn bevy_lunex entities on state transitions.

# Constraints

- Only the in-game HUD (clock/stats display) is in scope. Menu, outcome screens, power bar, and aim line are NOT modified.
- No changes to game physics, clock calculations, or any pure functions.
- Must remain compatible with the existing `GameState::Paused` / `GameState::Playing` flow.
- Must not break existing tests (unit tests for calculate_* and format_* functions).
- bevy_lunex is added as a new dependency — this is an intentional, user-requested deviation from PRD-0009's "no third-party UI crates" stance.

# References to Code

- `src/game/player/player_clock.rs` — Player HUD spawn + text update systems (lines 71–123)
- `src/game/observer/mod.rs` — Observer HUD spawn + text update systems (lines 20–63)
- `src/game/levels/mod.rs` — Calls `spawn_player_clock` and `spawn_observer_clock` during level setup
- `src/game/shared/types.rs` — `PlayerHud`, `Clock`, `VelocityGamma`, `GravitationalGamma` components
- `src/menu/mod.rs` — Existing bevy_ui menu (NOT modified)
- `assets/fonts/HackNerdFontMono-Regular.ttf` — Shared font asset

# Non-Goals (MVP)

- Migrating menu or outcome screens to bevy_lunex (separate PRD if desired)
- Animating HUD elements (hover states, transitions, glow pulses)
- Adding new data readouts beyond the existing five (t_p, γ_v, γ_g, v, t_o)
- Improving the power bar or aim line visuals (separate PRD)
- Creating a settings/options screen
- Adding gamepad/keyboard navigation for HUD elements (HUD is display-only)

# History

## 2026-02-08 — T-001 Completed
- **Task**: Add bevy_lunex dependency to Cargo.toml
- **Status**: ✅ Done
- **Changes**:
  - Added `bevy_lunex = "0.6"` to `[dependencies]` in `Cargo.toml`
  - Added `bevy_lunex = { version = "0.6", features = ["wasm"] }` to `[target.'cfg(target_arch = "wasm32")'.dependencies]` for WASM support
  - `Cargo.lock` updated with bevy_lunex and its transitive dependencies
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal change (two lines added to Cargo.toml), consistent with existing dependency style.

## 2026-02-08 — T-002 Completed
- **Task**: Create HUD module with bevy_lunex layout root and plugin
- **Status**: ✅ Done
- **Changes**:
  - Created `src/game/hud/mod.rs` with `HudPlugin` that spawns a `UiLayoutRoot::new_2d()` + `UiFetchFromCamera::<0>` entity on `OnEnter(AppState::InGame)` and despawns it on `OnExit(AppState::InGame)`.
  - Added `HudRoot` marker component and `GameItem` component to the layout root entity for lifecycle management.
  - Registered `pub mod hud` in `src/game/mod.rs` and added `HudPlugin` to `GamePlugin`.
  - Added `UiSourceCamera::<0>` to the camera spawn in `src/shared/types.rs` so `UiFetchFromCamera::<0>` can fetch viewport dimensions.
  - Added `UiLunexPlugins` in `src/main.rs` (not inside `GamePlugin`) so that bevy_lunex's cursor/picking systems have access to window resources from `DefaultPlugins`. This avoids panics in headless tests that use `MinimalPlugins` + `GamePlugin`.
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. `UiLunexPlugins` is registered in `main.rs` rather than in `HudPlugin` to respect separation of concerns — the plugin group requires `DefaultPlugins` resources, so it belongs at the app level.

## 2026-02-08 — T-003 Completed
- **Task**: Build bottom-anchored HUD bar with two segmented panels
- **Status**: ✅ Done
- **Changes**:
  - Expanded `src/game/hud/mod.rs` to build the full HUD layout tree under the existing `UiLayoutRoot`:
    - Bottom HUD bar at 88–100% screen height using `UiLayout::boundary()`
    - Left player panel (1–59% width) with semi-transparent dark background (`srgba(0.1, 0.1, 0.15, 0.85)`)
    - Right observer panel (64–99% width) with matching background
    - Cyan border accent (`srgba(0.3, 0.8, 1.0, 0.7)`) top-line on each panel
    - Placeholder text labels: `t_p`, `γ_v`, `γ_g`, `v` in left panel; `t_o` in right panel
    - All entities marked with `GameItem` for lifecycle management
  - Added marker components: `HudBar`, `PlayerPanel`, `ObserverPanel`
  - Used `ChildSpawnerCommands` (Bevy 0.18 API) for helper spawn functions
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal changes to single file, consistent with existing patterns.

## 2026-02-08 — T-004 Completed
- **Task**: Create or source HUD chrome sprite assets
- **Status**: ✅ Done
- **Changes**:
  - Created `assets/sprites/hud/panel.png` — a 48×48 programmatic 9-slice sprite with dark interior (`rgba(26, 26, 38, 217)`) and cyan border glow (`rgba(77, 204, 255, 179)`) with 4px border region
  - Created `assets/sprites/hud/border_accent.png` — an 8×8 solid cyan accent sprite (available for future use)
  - Updated `src/game/hud/mod.rs` to load `panel.png` and render panel backgrounds using `SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(4.0), .. })` instead of plain `Sprite::default()` with `UiColor`
  - Removed separate `spawn_panel_border` function — the border is now baked into the 9-slice panel sprite
  - Removed unused `PANEL_BG` and `BORDER_ACCENT` color constants
  - Added `PANEL_SPRITE` path constant and `PANEL_BORDER_PX` for 9-slice border width
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal changes to existing code, consistent with bevy_lunex 9-slice patterns documented in the library's README.

## 2026-02-08 — T-005 Completed
- **Task**: Wire player stats into left HUD panel
- **Status**: ✅ Done
- **Changes**:
  - Added four marker components in `src/game/hud/mod.rs`: `HudPlayerTime`, `HudVelocityGamma`, `HudGravGamma`, `HudVelocityFraction`
  - Updated `spawn_player_labels` to attach a unique marker component to each label entity (replacing the generic loop)
  - Added `player_hud_text_update` system that reads `Clock`, `VelocityGamma`, `GravitationalGamma` from the `PlayerHud` entity and `Velocity` from the `Player` entity, then writes formatted values into the four individual `Text2d` labels
  - Registered `player_hud_text_update` in `GamePlugin` (running after `player_clock_update` in the `Running` state)
  - Added imports for `Player`, `Clock`, `VelocityGamma`, `GravitationalGamma`, `Velocity`, `PlayerHud`, `C`, and `format_velocity_fraction` to the HUD module
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal changes, consistent with existing patterns. The old `player_clock_text_update` system remains untouched (it still updates the old `Text`-based HUD entity); removal is deferred to T-007.

## 2026-02-08 — T-006 Completed
- **Task**: Wire observer clock into right HUD panel
- **Status**: ✅ Done
- **Changes**:
  - Added `HudObserverTime` marker component in `src/game/hud/mod.rs`
  - Attached `HudObserverTime` marker to the observer label entity in `spawn_observer_labels`
  - Added `observer_hud_text_update` system that reads `Clock` from the `Observer` entity and writes the formatted time into the `HudObserverTime` `Text2d` component using the existing `format_observer_time` pure function
  - Registered `observer_hud_text_update` in `GamePlugin` (running after `observer_clock_update` in the `Running` state)
  - Added imports for `Observer` and `format_observer_time` from the observer module
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal changes to two files, consistent with the pattern established in T-005. The old `observer_clock_text_update` system remains untouched; removal is deferred to T-007.

## 2026-02-08 — T-007 Completed
- **Task**: Remove old top-positioned HUD spawn code
- **Status**: ✅ Done
- **Changes**:
  - Removed `Text` and `Node` fields from `PlayerClockBundle` in `src/game/player/player_clock.rs` — bundle is now data-only (Clock, VelocityGamma, GravitationalGamma, PlayerHud, Player, GameItem)
  - Simplified `spawn_player_clock` to take only `&mut Commands` (no `asset_server`) and spawn just the data bundle
  - Removed `player_clock_text_update` system from `src/game/player/player_clock.rs` (replaced by `player_hud_text_update` in `src/game/hud/mod.rs`)
  - Removed `Text` and `Node` fields from `ObserverClockBundle` in `src/game/observer/mod.rs` — bundle is now data-only
  - Simplified `spawn_observer_clock` to take only `&mut Commands` and spawn just the data bundle
  - Removed `observer_clock_text_update` system from `src/game/observer/mod.rs` (replaced by `observer_hud_text_update` in `src/game/hud/mod.rs`)
  - Updated `src/game/levels/mod.rs` to call the simplified spawn functions (no `asset_server` arg)
  - Removed system registrations and imports for `player_clock_text_update` and `observer_clock_text_update` from `src/game/mod.rs`
  - Updated `tests/e2e_velocity_hud.rs` to verify data components (VelocityGamma, Velocity) on the PlayerHud entity instead of querying removed `Text` component
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. All pure functions (calculate_*, format_*) and clock update systems (player_clock_update, observer_clock_update) left untouched. Only Node-based UI spawning and text-update systems removed.

## 2026-02-08 — T-008 Completed
- **Task**: Mark HUD entities with GameItem for lifecycle management
- **Status**: ✅ Done
- **Changes**:
  - Verified all HUD entities (HudRoot, HudBar, PlayerPanel, ObserverPanel, and all 5 text labels) already include `GameItem` component — this was done proactively during T-002 through T-006
  - Updated outdated comment in `src/game/levels/mod.rs` `despawn_level` function: replaced "game entities do not have children" with accurate note that Bevy 0.18 `despawn()` recursively despawns children
  - Confirmed `despawn_level` (GameItem-based) and `despawn_hud_root` (HudRoot-based) both correctly handle HUD cleanup with no double-despawn panics
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal change (one comment update), consistent with existing patterns.

## 2026-02-08 — T-009 Completed
- **Task**: Verify WASM build and test cross-platform
- **Status**: ✅ Done
- **Changes**:
  - Identified `getrandom` v0.2 (transitive via `nalgebra` → `rand` → `rand_core`) and v0.3 (transitive via `bevy_lunex` → `rand`) missing WASM features, causing `cargo check --target wasm32-unknown-unknown` to fail
  - Added `getrandom` v0.2 with `"js"` feature and v0.3 with `"wasm_js"` feature to `[target.'cfg(target_arch = "wasm32")'.dependencies]` in `Cargo.toml`
  - Verified `cargo check --target wasm32-unknown-unknown` passes
  - Verified `cargo make build-web` (full trunk release build) succeeds
  - UAT passed: 230 tests run, 230 passed, 0 skipped (`cargo make uat` exit code 0)
- **Constitution Compliance**: No violations. Minimal change (two lines added to Cargo.toml), consistent with existing dependency style.

## 2026-02-08 — T-010 Completed
- **Task**: Run full CI and fix any clippy/fmt issues
- **Status**: ✅ Done
- **Changes**:
  - Ran `cargo make ci` (fmt-check + clippy + nextest): all checks passed with zero warnings and zero errors
  - Ran `cargo make uat`: 230 tests run, 230 passed, 0 skipped (exit code 0)
  - No clippy or fmt issues found in the new HUD code — previous tasks maintained clean code throughout
  - No code changes required; this task is a verification-only gate
- **Constitution Compliance**: No violations. No code changes made.

## 2026-02-08 — uat-001 Verification
- **UAT**: HUD renders at bottom of screen with chrome panels
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_structure.rs` with two tests:
    - `hud_spawns_with_bottom_bar_and_chrome_panels`: Verifies HudRoot, HudBar, PlayerPanel (with Sprite), and ObserverPanel (with Sprite) all exist after entering InGame.
    - `hud_hierarchy_is_correct`: Verifies HudBar is a child of HudRoot, and both PlayerPanel and ObserverPanel are children of HudBar, confirming the bottom-anchored layout hierarchy.
  - Full UAT passed: 232 tests run, 232 passed, 0 skipped (`cargo make uat` exit code 0).
