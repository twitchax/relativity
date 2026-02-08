---
id: PRD-0010
title: "Improve HUD: Bottom-Anchored Chrome with bevy_lunex"
status: draft
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
    uat_status: unverified
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
    status: todo
    notes: "Add bevy_lunex = \"0.6\" to [dependencies]. Add wasm feature for WASM target in [target.'cfg(target_arch = \"wasm32\")'.dependencies]."
  - id: T-002
    title: "Create HUD module with bevy_lunex layout root and plugin"
    priority: 2
    status: todo
    notes: "Create src/game/hud/ module. Add a HudPlugin that registers UiLunexPlugins and spawns the HUD layout root (UiLayoutRoot::new_2d + UiFetchFromCamera). Register in game plugin."
  - id: T-003
    title: "Build bottom-anchored HUD bar with two segmented panels"
    priority: 3
    status: todo
    notes: "Use UiLayout::boundary() or window() to position a bar at the bottom of the screen. Create left panel (player stats) and right panel (observer clock). Use Sprite with 9-slice for chrome background. Semi-transparent dark background (0.1, 0.1, 0.15, 0.85) with thin cyan/white border accent."
  - id: T-004
    title: "Create or source HUD chrome sprite assets"
    priority: 3
    status: todo
    notes: "Create minimal panel background sprites for 9-slice rendering (dark panel with border glow). Place in assets/sprites/hud/. Can be simple programmatic textures or hand-crafted PNGs."
  - id: T-005
    title: "Wire player stats into left HUD panel"
    priority: 4
    status: todo
    notes: "Display t_p, γ_v, γ_g, v as separate labeled readouts using Text2d + UiTextSize. Each on its own line or in a grid within the panel. Update player_clock_text_update to target new entities."
  - id: T-006
    title: "Wire observer clock into right HUD panel"
    priority: 4
    status: todo
    notes: "Display t_o in the right panel. Update observer_clock_text_update to target new entity."
  - id: T-007
    title: "Remove old top-positioned HUD spawn code"
    priority: 5
    status: todo
    notes: "Remove spawn_player_clock and spawn_observer_clock old Node-based spawning. Keep pure functions (calculate_*, format_*) untouched."
  - id: T-008
    title: "Mark HUD entities with GameItem for lifecycle management"
    priority: 5
    status: todo
    notes: "Ensure all new HUD entities include the GameItem component so they are properly despawned on state transitions."
  - id: T-009
    title: "Verify WASM build and test cross-platform"
    priority: 6
    status: todo
    notes: "Run cargo make build-web. Ensure bevy_lunex wasm feature is enabled for wasm32 target. Test in browser."
  - id: T-010
    title: "Run full CI and fix any clippy/fmt issues"
    priority: 7
    status: todo
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
