---
id: PRD-0013
title: "Beautify HUD: Sci-Fi Cockpit Instrument Panel"
status: draft
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
- Follow the project constitution: minimal changes, DRY, separation of concerns

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
  uat_status: unverified
- id: uat-002
  name: "HUD bar remains anchored to the bottom 12% of the screen"
  command: cargo make uat
  uat_status: unverified
- id: uat-003
  name: "Player panel (left) and observer panel (right) are visually distinct with new panel art"
  command: cargo make uat
  uat_status: unverified
- id: uat-004
  name: "Labels use a display font; numeric values use the monospace font"
  command: cargo make uat
  uat_status: unverified
- id: uat-005
  name: "Value changes trigger a subtle visual feedback (color flash, glow pulse, or highlight)"
  command: cargo make uat
  uat_status: unverified
- id: uat-006
  name: "Gamma-based color shifting: readouts shift from cool (blue/cyan) toward warm (orange/red) as γ increases"
  command: cargo make uat
  uat_status: unverified
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
  status: todo
  notes: "Agent makes design choices. Decide on: panel background gradient/texture, border glow color & intensity, inner section dividers, and overall light/dark balance. Document decisions as code comments."
- id: T-002
  title: "Create or source new panel sprite assets"
  priority: 1
  status: todo
  notes: "Create new 9-slice panel sprites: a main panel background with richer detail (gradient, inner bevel, subtle noise), and optional accent/divider sprites. Keep assets small (<10 KB each). Consider a separate panel style for player vs. observer to add visual variety."
- id: T-003
  title: "Add a display font for labels/section headers"
  priority: 2
  status: todo
  notes: "Agent picks a sci-fi display font (e.g., Orbitron, Rajdhani, Exo 2, or similar OFL-licensed font). Use it for label prefixes (t_p, γ_v, etc.) or section headers. Keep Hack Nerd Font Mono for numeric values. Add the .ttf to assets/fonts/."
- id: T-004
  title: "Redesign panel layout with visual hierarchy and grouping"
  priority: 2
  status: todo
  notes: "Improve the internal layout of each panel: add section labels/headers, visual separators between readouts, and better vertical spacing. Consider grouping related stats (e.g., γ_v and γ_g together). Maintain bevy_lunex relative positioning."
- id: T-005
  title: "Add decorative elements: icons, divider lines, or accent graphics"
  priority: 3
  status: todo
  notes: "Agent decides what decorative elements to add. Possibilities: small icons next to each stat (clock icon for time, speedometer for velocity), thin horizontal dividers, corner accent flourishes, or a subtle panel title bar. Keep it tasteful and not cluttered."
- id: T-006
  title: "Implement gamma-based dynamic color shifting on readouts"
  priority: 2
  status: todo
  notes: "Readout text color shifts based on the displayed value's intensity. For γ_v and γ_g: interpolate from cyan (γ ≈ 1) toward orange/red (γ ≥ 3), matching the trail system's color language. For velocity: shift from cool to warm as v approaches c. Use UiColor updates in the existing HUD update systems."
- id: T-007
  title: "Implement subtle value-change animations (pulse/flash on update)"
  priority: 3
  status: todo
  notes: "When a displayed value changes significantly, briefly flash the text brighter or pulse the label's alpha/scale. Use a small Bevy timer or tween component. Keep the effect subtle (100-300ms duration, slight brightness boost). Avoid per-frame allocations."
- id: T-008
  title: "Polish: panel glow, border animations, and final tuning"
  priority: 3
  status: todo
  notes: "Final visual pass: adjust glow intensity, tweak font sizes for balance, fine-tune spacing, and ensure the HUD looks cohesive with the game's background and gravity grid. Verify the HUD doesn't obscure important gameplay area."
- id: T-009
  title: "Verify all existing tests pass and HUD respawns correctly"
  priority: 1
  status: todo
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

---
