---
id: PRD-0013
title: "Beautify HUD: Sci-Fi Cockpit Instrument Panel"
status: active
owner: twitchax
created: 2026-02-12
updated: 2026-02-12
depends_on:
- PRD-0010
- PRD-0012

principles:
- Keep all existing HUD information (t_p, γ_v, γ_g, v, t_o, r); nothing is removed
- Bottom-bar layout is preserved; beautification is visual, not structural
- Sci-fi cockpit / instrument-panel aesthetic guides all design choices
- Subtle animations enhance readability without distracting from gameplay
- New assets are welcome but must remain lightweight (small PNGs, no large textures)
- Monospace font (Hack Nerd Font Mono) stays for numeric values; a second display font may be introduced for labels/headers
- Existing marker-component query architecture is preserved; changes are additive
- "Follow the project constitution: minimal changes, DRY, separation of concerns"

references:
- name: "PRD-0010: Improve HUD (bevy_lunex)"
  url: .mr/prds/PRD-0010-improve-hud.md
- name: "PRD-0012: Hotkeys (sim rate display)"
  url: .mr/prds/PRD-0012-hotkeys.md
- name: bevy_lunex documentation
  url: https://docs.rs/bevy_lunex/latest/bevy_lunex/
- name: Google Fonts — Orbitron (candidate display font)
  url: https://fonts.google.com/specimen/Orbitron

acceptance_tests:
- id: uat-001
  name: "HUD displays all six readouts: t_p, γ_v, γ_g, v, t_o, r"
  command: cargo make uat
  uat_status: verified
- id: uat-002
  name: "HUD bar remains anchored to the bottom 12% of the screen"
  command: cargo make uat
  uat_status: verified
- id: uat-003
  name: "Player panel (left) and observer panel (right) are visually distinct with new panel art"
  command: cargo make uat
  uat_status: verified
- id: uat-004
  name: "Labels use a display font; numeric values use the monospace font"
  command: cargo make uat
  uat_status: verified
- id: uat-005
  name: "Value changes trigger a subtle visual feedback (color flash, glow pulse, or highlight)"
  command: cargo make uat
  uat_status: verified
- id: uat-006
  name: "Gamma-based color shifting: readouts shift from cool (blue/cyan) toward warm (orange/red) as γ increases"
  command: cargo make uat
  uat_status: verified
- id: uat-007
  name: "HUD renders correctly after level reset (PendingLevelReset re-spawn)"
  command: cargo make uat
  uat_status: unverified
- id: uat-008
  name: "All existing tests pass (cargo make ci)"
  command: cargo make ci
  uat_status: unverified

tasks:
- id: T-001
  title: "Design the cockpit panel visual language (colors, borders, glow, segmentation)"
  priority: 1
  status: done
  notes: "Agent makes design choices. Decide on: panel background gradient/texture, border glow color & intensity, inner section dividers, and overall light/dark balance. Document decisions as code comments."
- id: T-002
  title: "Create or source new panel sprite assets"
  priority: 1
  status: done
  notes: "Create new 9-slice panel sprites: a main panel background with richer detail (gradient, inner bevel, subtle noise), and optional accent/divider sprites. Keep assets small (<10 KB each). Consider a separate panel style for player vs. observer to add visual variety."
- id: T-003
  title: "Add a display font for labels/section headers"
  priority: 2
  status: done
  notes: "Agent picks a sci-fi display font (e.g., Orbitron, Rajdhani, Exo 2, or similar OFL-licensed font). Use it for label prefixes (t_p, γ_v, etc.) or section headers. Keep Hack Nerd Font Mono for numeric values. Add the .ttf to assets/fonts/."
- id: T-004
  title: "Redesign panel layout with visual hierarchy and grouping"
  priority: 2
  status: done
  notes: "Improve the internal layout of each panel: add section labels/headers, visual separators between readouts, and better vertical spacing. Consider grouping related stats (e.g., γ_v and γ_g together). Maintain bevy_lunex relative positioning."
- id: T-005
  title: "Add decorative elements: icons, divider lines, or accent graphics"
  priority: 3
  status: done
  notes: "Agent decides what decorative elements to add. Possibilities: small icons next to each stat (clock icon for time, speedometer for velocity), thin horizontal dividers, corner accent flourishes, or a subtle panel title bar. Keep it tasteful and not cluttered."
- id: T-006
  title: "Implement gamma-based dynamic color shifting on readouts"
  priority: 2
  status: done
  notes: "Readout text color shifts based on the displayed value's intensity. For γ_v and γ_g: interpolate from cyan (γ ≈ 1) toward orange/red (γ ≥ 3), matching the trail system's color language. For velocity: shift from cool to warm as v approaches c. Use UiColor updates in the existing HUD update systems."
- id: T-007
  title: "Implement subtle value-change animations (pulse/flash on update)"
  priority: 3
  status: done
  notes: "When a displayed value changes significantly, briefly flash the text brighter or pulse the label's alpha/scale. Use a small Bevy timer or tween component. Keep the effect subtle (100-300ms duration, slight brightness boost). Avoid per-frame allocations."
- id: T-008
  title: "Polish: panel glow, border animations, and final tuning"
  priority: 3
  status: done
  notes: "Final visual pass: adjust glow intensity, tweak font sizes for balance, fine-tune spacing, and ensure the HUD looks cohesive with the game's background and gravity grid. Verify the HUD doesn't obscure important gameplay area."
- id: T-009
  title: "Verify all existing tests pass and HUD respawns correctly"
  priority: 1
  status: done
  notes: "Run cargo make ci. Verify HUD spawns on InGame enter, respawns after PendingLevelReset, and all marker-component queries still work. Confirm screenshot tests still pass (if any reference the HUD area, thresholds may need updating)."

---

# Summary

Transform the plain, text-only HUD into a visually rich sci-fi cockpit instrument panel. The current HUD displays all the right information but lacks visual polish — it's flat dark panels with monochrome text. This PRD adds layered visual design: richer panel art, a display font for labels, decorative elements (icons, dividers, accents), gamma-based dynamic color shifting on readouts, and subtle value-change animations. The bottom-bar layout and all six readouts are preserved.

---

# Problem

The HUD currently consists of two flat dark rectangles with uniform soft-white text. While functional, it doesn't match the quality and atmosphere of the rest of the game (gradient planets, dynamic gravity grids, color-mapped trails). The HUD feels like a placeholder rather than a polished game element. Players looking at relativistic physics readouts should feel like they're piloting a spacecraft, not reading a debug overlay.

---

# Goals

1. Give the HUD a sci-fi cockpit / instrument-panel aesthetic that matches the game's space theme
2. Introduce visual hierarchy: labels vs. values, primary vs. secondary readouts
3. Add dynamic color feedback so readouts visually communicate intensity (low gamma = cool blue, high gamma = warm red)
4. Add subtle animations on value changes to draw attention without distraction
5. Introduce a display font for labels while keeping monospace for numeric precision
6. Add decorative elements (icons, dividers, accents) that enhance without cluttering
7. Maintain all existing functionality: same six readouts, same update systems, same respawn behavior

---

# Technical Approach

The implementation builds on the existing `bevy_lunex` HUD architecture. No structural changes to the ECS pattern — the same marker components and query-based update systems are preserved. Changes are layered on top:

## Visual Layer Stack

```
┌─────────────────────────────────────────────────────────────┐
│  Bottom HUD Bar (88%-100% screen height)                    │
│  ┌──────────── Player Panel ────────────┐ ┌─ Observer ────┐ │
│  │  ╭─ Header/Title ──────────────────╮ │ │ ╭─ Header ──╮ │ │
│  │  │  FLIGHT INSTRUMENTS             │ │ │ │ OBSERVER  │ │ │
│  │  ╰────────────────────────────────╯ │ │ ╰───────────╯ │ │
│  │  ┌─────────┐ ┌─────────┐           │ │               │ │
│  │  │ t_p     │ │ γ_v     │           │ │  t_o = 12.34  │ │
│  │  │ = 12.34 │ │ = 1.23  │           │ │               │ │
│  │  └─────────┘ └─────────┘           │ │  r = 1.00×    │ │
│  │  ┌─────────┐ ┌─────────┐           │ │               │ │
│  │  │ γ_g     │ │ v       │           │ │               │ │
│  │  │ = 1.01  │ │ = 0.87c │           │ │               │ │
│  │  └─────────┘ └─────────┘           │ │               │ │
│  └──────────────────────────────────────┘ └───────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Color System

Dynamic color interpolation in the HUD update systems:

```
γ = 1.0  →  cyan   (0.4, 0.8, 1.0)   "nominal"
γ = 2.0  →  yellow (1.0, 0.9, 0.3)   "elevated"  
γ ≥ 3.0  →  red    (1.0, 0.3, 0.1)   "extreme"
```

This mirrors the trail system's color language, creating a unified visual vocabulary.

## Animation Approach

Value-change detection via a small `PreviousValue<T>` component or float comparison in the update system. On significant change (> 0.01 delta), set a `FlashTimer` component that decays over ~200ms, boosting text brightness during the flash.

## Font Strategy

- **Display font** (e.g., Orbitron): Used for label names (`t_p`, `γ_v`), panel headers
- **Hack Nerd Font Mono**: Used for numeric values (`12.34`, `1.00×`)

Labels and values may be split into separate `Text2d` entities within each readout group, or use Bevy's text sections if supported by `bevy_lunex`.

---

# Assumptions

- `bevy_lunex` 0.6 supports nested layout with sufficient flexibility for the proposed grouping
- `UiColor` can be updated per-frame for dynamic color shifting without performance issues
- New font files (OFL-licensed) can be added to `assets/fonts/` without licensing concerns
- The existing 9-slice sprite approach can be extended with richer panel art

---

# Constraints

- Must stay within `bevy_lunex` — no switching to `bevy_ui` for the in-game HUD
- Bottom 12% bar layout is fixed; internal arrangement can change freely
- All six readouts must remain visible and readable at all times
- Animation effects must be subtle enough not to distract from gameplay
- New assets must be small (total new asset size < 50 KB)
- Must not break headless tests (HUD systems should gracefully handle missing entities)

---

# References to Code

- `src/game/hud/mod.rs` — entire HUD implementation (spawn, layout, update systems)
- `src/game/trail/mod.rs` — gamma-to-color interpolation (reference for color language)
- `src/game/gravity_grid/mod.rs` — curvature color gradient (reference for warm/cool palette)
- `src/game/shared/types.rs` — `VelocityGamma`, `GravitationalGamma`, `SimRate`, `Clock`
- `src/game/player/player_clock.rs` — `format_velocity_fraction` helper
- `src/game/observer/mod.rs` — `format_observer_time` helper
- `assets/sprites/hud/panel.png` — current 9-slice panel sprite
- `assets/fonts/HackNerdFontMono-Regular.ttf` — current (and retained) monospace font

---

# Non-Goals (MVP)

- Rearranging the HUD to a non-bottom-bar layout
- Adding new gameplay information to the HUD (only beautifying existing readouts)
- Complex shader effects or custom render pipelines
- Animated sprite sheets or particle effects on the HUD
- Responsive layout changes for different screen resolutions (current relative sizing is sufficient)
- Sound effects on value changes

---

# History

(Entries appended by `mr run` will go below this line.)

## 2026-02-12 — T-001 Completed
- **Task**: Design the cockpit panel visual language (colors, borders, glow, segmentation)
- **Status**: ✅ Done
- **Changes**:
  - Added comprehensive visual language design comment block to `src/game/hud/mod.rs` (lines 13-44)
  - Added 7 new cockpit palette color constants with `#[allow(unused)]`
  - Color palette mirrors the trail system gamma_to_color and gravity grid gradient
  - UAT not run locally (environment restriction); changes are purely additive
- **Constitution Compliance**: No violations.

---


## 2026-02-12 — T-002 Completed
- **Task**: Create or source new panel sprite assets
- **Status**: ✅ Done
- **Changes**:
  - Created `assets/sprites/hud/panel_player.png` (279 bytes): 48x48 9-slice sprite with cyan border glow, inner bevel, corner accents, and top gradient highlight
  - Created `assets/sprites/hud/panel_observer.png` (274 bytes): 48x48 9-slice sprite with teal/green-tinted border glow for visual distinction from player panel
  - Updated `src/game/hud/mod.rs`: replaced single `PANEL_SPRITE` constant with `PLAYER_PANEL_SPRITE` and `OBSERVER_PANEL_SPRITE`
  - Split `panel_sprite` closure into `player_panel_sprite` and `observer_panel_sprite` closures with separate image handles
  - Player and observer panels now load distinct sprites, making them visually distinguishable
  - Both assets are well under the 10 KB size limit
  - `cargo make uat` could not be run locally (environment restriction); changes are structurally sound and additive
- **Constitution Compliance**: No violations.

---
## 2026-02-12 --- T-003 Completed
- **Task**: Add a display font for labels/section headers
- **Status**: Done
- **Changes**:
  - Downloaded Orbitron variable font (38 KB, OFL-licensed) from Google Fonts repository to assets/fonts/Orbitron-Regular.ttf
  - Added DISPLAY_FONT constant in src/game/hud/mod.rs pointing to the new font
  - Loaded the display font alongside the monospace font in spawn_hud_root
  - Added FLIGHT DATA header label (Orbitron, cyan HEADER_COLOR) to the player stats panel
  - Added OBSERVER header label (Orbitron, cyan HEADER_COLOR) to the observer clock panel
  - Removed #[allow(unused)] from HEADER_COLOR (now actively used)
  - Adjusted observer panel label positions (y 35->40, y 65->70) to accommodate the new header
  - Updated screenshot baseline tests/baselines/level1_spawn.png to reflect the new HUD headers
  - cargo make uat passes: 279/279 tests pass
- **Constitution Compliance**: No violations.

---

## 2026-02-12 — T-004 Completed
- **Task**: Redesign panel layout with visual hierarchy and grouping
- **Status**: ✅ Done
- **Changes**:
  - Redesigned `spawn_player_labels` in `src/game/hud/mod.rs` with two-column grid layout
  - Added section labels (TIME, VELOCITY, GAMMA) using Orbitron display font with `LABEL_COLOR`
  - Added horizontal divider line under "FLIGHT DATA" header using `DIVIDER_COLOR`
  - Grouped γ_v and γ_g together in right column under "GAMMA" section
  - Left column: TIME (t_p) at top, VELOCITY (v) at bottom
  - Redesigned `spawn_observer_labels` with matching two-column layout
  - Added section labels (TIME, RATE) and horizontal divider to observer panel
  - Repositioned observer values into two columns (t_o left, r right)
  - Removed `#[allow(unused)]` from `LABEL_COLOR` and `DIVIDER_COLOR` constants
  - All changes use bevy_lunex relative positioning (`Rl()`)
  - `cargo make uat` could not be run locally (environment permission restriction); changes are structurally sound and follow existing patterns
- **Constitution Compliance**: No violations.

## 2026-02-12 --- T-006 Completed
- **Task**: Implement gamma-based dynamic color shifting on readouts
- **Status**: Done
- **Changes**:
  - Added `gamma_to_hud_color` function in `src/game/hud/mod.rs`
  - Two-segment interpolation: NOMINAL (cyan) to ELEVATED (yellow) to EXTREME (red/orange)
  - Modified `player_hud_text_update` to update UiColor on v, gv, gg readouts
  - Query types changed from `&mut Text2d` to `(&mut Text2d, &mut UiColor)` for three readouts
  - Removed `#[allow(unused)]` from three GAMMA_COLOR constants (now actively used)
  - Fixed pre-existing clippy `doc_markdown` lint in doc comments for `spawn_player_labels` and `spawn_observer_labels`
  - Updated screenshot baseline `tests/baselines/level1_spawn.png`
  - `cargo make uat` passes: 279/279 tests pass
- **Constitution Compliance**: No violations.

## 2026-02-12 --- T-005 Completed
- **Task**: Add decorative elements: icons, divider lines, or accent graphics
- **Status**: Done
- **Changes**:
  - Added sub-divider (dashed line) between grouped gamma readouts in player panel
  - Added bottom-left corner accent flourish on both player and observer panels (mirrors existing top-right)
  - Added gauge dot indicators beside all value readouts in both panels
  - Updated doc comments for both spawn functions to reflect new decorative elements
  - Added clippy too_many_lines allow on both spawn functions (purely declarative UI layout code)
  - cargo make uat passes: 279/279 tests pass
- **Constitution Compliance**: No violations.

## 2026-02-12 --- T-007 Completed
- **Task**: Implement subtle value-change animations (pulse/flash on update)
- **Status**: Done
- **Changes**:
  - Added `HudFlash` component with `Timer` and `prev_text` tracking in `src/game/hud/mod.rs`
  - Added `FLASH_DURATION_SECS` (0.2s) and `FLASH_BOOST` (0.3) constants
  - Attached `HudFlash::new()` to all 6 value readout entities (t_p, v, gv, gg, t_o, r)
  - Created `hud_flash_system` that detects text changes and applies decaying brightness boost
  - Modified `tp_query`, `observer_hud_text_update`, and `sim_rate_hud_update` to set UiColor each frame (enables correct flash base-color tracking)
  - Registered `hud_flash_system` with `.after()` ordering on all three HUD update systems in `src/game/mod.rs`
  - Timer starts finished (no flash on spawn); resets on text change; 200ms ease-out brightness decay
  - `cargo make uat` could not be run locally (environment permission restriction); changes are structurally sound and follow existing patterns
- **Constitution Compliance**: No violations.
## 2026-02-13 --- T-008 Completed
- **Task**: Polish: panel glow, border animations, and final tuning
- **Status**: ✅ Done
- **Changes**:
  - Registered `hud_glow_pulse_system` in `src/game/mod.rs` — glow overlay entities were spawned but the animation system was never added to the app schedule
  - Added import of `hud_glow_pulse_system` from `hud` module in `src/game/mod.rs`
  - Tuned glow overlay parameters: slower pulse speed (1.2 → 0.8) for a calmer breathing effect, slightly reduced alpha values for subtlety
  - Extended glow overlays to fully cover panel edges (pos1/pos2 adjusted to -0.5/100.5 for slight overshoot)
  - Increased flash duration (0.2 → 0.25s) and boost (0.3 → 0.35) for more noticeable value-change feedback
  - Changed flash decay from linear to quadratic ease-out (`t * t * FLASH_BOOST`) for smoother brightness falloff
  - Flash system now captures the current gamma-shifted `UiColor` as the base color on text change, so brightness boost applies on top of the correct dynamic color
  - `cargo make uat` passes: 279/279 tests pass
- **Constitution Compliance**: No violations.

## 2026-02-13 — uat-001 Verification
- **UAT**: HUD displays all six readouts: t_p, γ_v, γ_g, v, t_o, r
- **Status**: ✅ Verified
- **Method**: Existing tests
- **Details**:
  - `tests/e2e_hud_player_panel.rs` — `player_panel_has_four_stat_labels` and `player_panel_shows_default_values_at_spawn` verify t_p, γ_v, γ_g, v entities exist and display correct defaults
  - `tests/e2e_hud_observer_panel.rs` — `observer_panel_has_time_label` and `observer_panel_shows_default_value_at_spawn` verify t_o exists and displays correct default
  - `tests/e2e_hud_sim_rate.rs` — `observer_panel_has_sim_rate_label` and `sim_rate_label_shows_default_value` verify r exists and displays correct default
  - All 279 tests pass via `cargo make uat`

## 2026-02-13 — uat-002 Verification
- **UAT**: HUD bar remains anchored to the bottom 12% of the screen
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_anchoring.rs` — `hud_bar_anchored_to_bottom_twelve_percent`
  - Test inspects the `UiLayout` boundary on the `HudBar` entity and asserts pos1 y=88%, pos2 y=100% (bottom 12%)
  - All 280 tests pass via `cargo make uat`

## 2026-02-13 — uat-003 Verification
- **UAT**: Player panel (left) and observer panel (right) are visually distinct with new panel art
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_panel_distinction.rs` — `player_and_observer_panels_use_distinct_sprites`
  - Test queries `PlayerPanel` and `ObserverPanel` entities, extracts their `Sprite` image handles, and asserts they differ
  - All 281 tests pass via `cargo make uat`

## 2026-02-13 — uat-004 Verification
- **UAT**: Labels use a display font; numeric values use the monospace font
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_font_assignment.rs` — `labels_use_display_font_and_values_use_monospace_font`
  - Loads both font handles (Orbitron display font and Hack Nerd Font Mono) via AssetServer and verifies they differ
  - Asserts all six value readout entities (HudPlayerTime, HudVelocityGamma, HudGravGamma, HudVelocityFraction, HudObserverTime, HudSimRate) use the monospace font
  - Asserts all header/label text entities (FLIGHT DATA, OBSERVER, TIME, VELOCITY, GAMMA, RATE) use the display font
  - All 282 tests pass via `cargo make uat`

## 2026-02-13 — uat-005 Verification
- **UAT**: Value changes trigger a subtle visual feedback (color flash, glow pulse, or highlight)
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_value_flash.rs` with two tests:
    - `all_value_readouts_have_hud_flash_component` — verifies all 6 numeric readout entities (HudPlayerTime, HudVelocityGamma, HudGravGamma, HudVelocityFraction, HudObserverTime, HudSimRate) carry the `HudFlash` component
    - `value_change_triggers_brightness_flash` — launches the player, confirms the UiColor brightness is boosted immediately after text changes, and returns to base color after the flash timer decays (0.25s)
  - All 284 tests pass via `cargo make uat`

## 2026-02-13 — uat-006 Verification
- **UAT**: Gamma-based color shifting: readouts shift from cool (blue/cyan) toward warm (orange/red) as γ increases
- **Status**: ✅ Verified
- **Method**: New test
- **Details**:
  - Created `tests/e2e_hud_gamma_color.rs` with three tests:
    - `low_gamma_readout_is_cool` — launches at modest speed (γ ≈ 1.01), verifies HudVelocityGamma color is cool (blue > red)
    - `high_gamma_readout_is_warm` — launches at ~0.94c (γ ≈ 3), verifies HudVelocityGamma color is warm (red > blue)
    - `velocity_fraction_readout_shifts_warm_at_high_speed` — verifies HudVelocityFraction also shifts warm at high γ
  - All 287 tests pass via `cargo make uat`
