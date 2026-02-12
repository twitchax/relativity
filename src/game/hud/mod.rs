use std::time::Duration;

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::game::observer::{format_observer_time, Observer};
use crate::game::player::player_clock::format_velocity_fraction;
use crate::game::player::shared::Player;
use crate::game::shared::constants::C;
use crate::game::shared::types::{Clock, GameItem, GravitationalGamma, PendingLevelReset, PlayerHud, SimRate, Velocity, VelocityGamma};
use crate::shared::state::{AppState, GameState};
use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};

// ────────────────────────────────────────────────────────────────────
// Cockpit Panel Visual Language (T-001)
// ────────────────────────────────────────────────────────────────────
//
// Design: sci-fi cockpit instrument panel aesthetic.
//
// **Palette** (mirrors the trail / gravity-grid color language):
//   - Panel background:  deep navy-black (0.04, 0.06, 0.12)
//   - Panel border glow: cyan          (0.30, 0.70, 1.00)
//   - Label text:        muted cyan    (0.50, 0.72, 0.85) — display font
//   - Value text:        bright white  (0.90, 0.95, 1.00) — monospace font
//   - Section headers:   brighter cyan (0.45, 0.82, 1.00)
//   - Divider lines:     dim cyan      (0.25, 0.50, 0.70, 0.40)
//
// **Dynamic color shifting** (applied to value readouts):
//   γ ≈ 1.0  →  cyan   (0.40, 0.80, 1.00)  — nominal
//   γ ≈ 2.0  →  yellow (1.00, 0.90, 0.30)  — elevated
//   γ ≥ 3.0  →  red    (1.00, 0.30, 0.10)  — extreme
//   Uses the same `gamma_to_color` blend as the trail system.
//
// **Light / dark balance**: near-black panels with bright colored
//   accents keep the HUD readable against the dark space backdrop
//   without washing out the game view.
//
// **Borders & glow**: panels use a 9-slice sprite with a subtle
//   cyan outer glow (matches the PANEL_BORDER_GLOW constant).
//   A second "border accent" sprite can be layered for extra detail.
//
// **Segmentation**: player panel (left 60%) vs observer panel
//   (right 35%) are visually distinct via slight tint variation.
//   Inner readouts are separated by thin horizontal divider lines.
// ────────────────────────────────────────────────────────────────────

// Paths.

/// 9-slice player panel sprite (dark with cyan border glow + inner bevel).
const PLAYER_PANEL_SPRITE: &str = "sprites/hud/panel_player.png";
/// 9-slice observer panel sprite (dark with teal border glow + inner bevel).
const OBSERVER_PANEL_SPRITE: &str = "sprites/hud/panel_observer.png";
/// Sci-fi display font (Orbitron, OFL-licensed) for panel headers and labels.
const DISPLAY_FONT: &str = "fonts/Orbitron-Regular.ttf";
/// Border pixel width in the panel sprites (must match the assets).
const PANEL_BORDER_PX: f32 = 4.0;

/// Duration of the brightness flash when a readout value changes.
const FLASH_DURATION_SECS: f32 = 0.2;

/// Maximum brightness boost applied at the start of a flash.
const FLASH_BOOST: f32 = 0.3;

// Colors — cockpit palette.

/// Soft white used for numeric value readouts (monospace font).
const TEXT_COLOR: Color = Color::srgba(0.9, 0.95, 1.0, 1.0);

/// Muted cyan for section label headers (display font).
const LABEL_COLOR: Color = Color::srgba(0.50, 0.72, 0.85, 1.0);

/// Brighter cyan for panel / section header text.
const HEADER_COLOR: Color = Color::srgba(0.45, 0.82, 1.00, 1.0);

/// Dim cyan for decorative divider lines.
const DIVIDER_COLOR: Color = Color::srgba(0.25, 0.50, 0.70, 0.50);

/// Cyan border glow tint (matches panel sprite outer glow).
const PANEL_BORDER_GLOW: Color = Color::srgba(0.30, 0.70, 1.00, 1.0);

/// Accent color for small corner and decorative flourish elements.
const ACCENT_COLOR: Color = Color::srgba(0.35, 0.65, 0.90, 0.60);

// Dynamic color-shift anchors (γ → readout text color).

/// Nominal color for γ ≈ 1.0 — calm cyan.
const GAMMA_COLOR_NOMINAL: Color = Color::srgba(0.40, 0.80, 1.00, 1.0);

/// Elevated color for γ ≈ 2.0 — warm yellow.
const GAMMA_COLOR_ELEVATED: Color = Color::srgba(1.00, 0.90, 0.30, 1.0);

/// Extreme color for γ ≥ 3.0 — hot red/orange.
const GAMMA_COLOR_EXTREME: Color = Color::srgba(1.00, 0.30, 0.10, 1.0);

// Pure functions.

#[must_use]
fn gamma_to_hud_color(gamma: f64) -> Color {
    #[allow(clippy::cast_possible_truncation)]
    let t = ((gamma - 1.0) / 2.0).clamp(0.0, 1.0) as f32;

    let (from, to, local_t) = if t < 0.5 {
        (GAMMA_COLOR_NOMINAL.to_srgba(), GAMMA_COLOR_ELEVATED.to_srgba(), t * 2.0)
    } else {
        (GAMMA_COLOR_ELEVATED.to_srgba(), GAMMA_COLOR_EXTREME.to_srgba(), (t - 0.5) * 2.0)
    };

    Color::srgba(
        from.red + local_t * (to.red - from.red),
        from.green + local_t * (to.green - from.green),
        from.blue + local_t * (to.blue - from.blue),
        1.0,
    )
}

// Marker components.

/// Marker component for the HUD layout root entity.
#[derive(Component, Default)]
pub struct HudRoot;

/// Marker for the bottom HUD bar container.
#[derive(Component, Default)]
pub struct HudBar;

/// Marker for the player stats panel (left).
#[derive(Component, Default)]
pub struct PlayerPanel;

/// Marker for the observer clock panel (right).
#[derive(Component, Default)]
pub struct ObserverPanel;

/// Marker for the player time (`t_p`) HUD label.
#[derive(Component)]
pub struct HudPlayerTime;

/// Marker for the velocity gamma (`γ_v`) HUD label.
#[derive(Component)]
pub struct HudVelocityGamma;

/// Marker for the gravitational gamma (`γ_g`) HUD label.
#[derive(Component)]
pub struct HudGravGamma;

/// Marker for the velocity fraction (v) HUD label.
#[derive(Component)]
pub struct HudVelocityFraction;

/// Marker for the observer time (`t_o`) HUD label.
#[derive(Component)]
pub struct HudObserverTime;

/// Marker for the simulation rate (`r`) HUD label.
#[derive(Component)]
pub struct HudSimRate;

/// Tracks previous text and drives a brightness-flash on value change.
///
/// Attached to each numeric readout entity. When the displayed text
/// changes, the timer resets and a brightness boost decays over
/// `FLASH_DURATION_SECS`.
#[derive(Component)]
pub struct HudFlash {
    timer: Timer,
    prev_text: String,
}

impl HudFlash {
    fn new() -> Self {
        let mut timer = Timer::from_seconds(FLASH_DURATION_SECS, TimerMode::Once);
        // Start finished so no flash fires on initial spawn.
        timer.tick(Duration::from_secs_f32(FLASH_DURATION_SECS));
        Self { timer, prev_text: String::new() }
    }
}

/// Marker for animated panel glow overlays with a breathing pulse.
///
/// Attached to slightly oversized, semi-transparent sprites behind
/// each panel. The pulse system modulates alpha with a slow sine
/// wave to create a subtle border glow effect.
#[derive(Component)]
pub struct HudGlow {
    color: Color,
    base_alpha: f32,
    amplitude: f32,
    speed: f32,
    offset: f32,
}

/// Plugin that spawns the HUD layout root.
///
/// Requires `UiLunexPlugins` to be registered at the app level
/// (alongside `DefaultPlugins`) so that picking/cursor systems
/// have the resources they need.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // No explicit OnExit despawn needed: the HudRoot has GameItem,
        // and despawn_level recursively cleans up all children.
        app.add_systems(OnEnter(AppState::InGame), spawn_hud_root)
            // Respawn the HUD after a level reset (PendingLevelReset despawns all GameItem entities).
            .add_systems(OnEnter(GameState::Paused), spawn_hud_root.run_if(resource_exists::<PendingLevelReset>));
    }
}

/// Spawns the `bevy_lunex` layout root and the bottom-anchored HUD bar
/// with player stats (left) and observer clock (right) panels.
fn spawn_hud_root(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/HackNerdFontMono-Regular.ttf");
    let display_font = asset_server.load(DISPLAY_FONT);
    let player_panel_image: Handle<Image> = asset_server.load(PLAYER_PANEL_SPRITE);
    let observer_panel_image: Handle<Image> = asset_server.load(OBSERVER_PANEL_SPRITE);

    let player_panel_sprite = || Sprite {
        image: player_panel_image.clone(),
        image_mode: SpriteImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(PANEL_BORDER_PX),
            ..Default::default()
        }),
        ..Default::default()
    };

    let observer_panel_sprite = || Sprite {
        image: observer_panel_image.clone(),
        image_mode: SpriteImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(PANEL_BORDER_PX),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Position the layout root at the camera center so bevy_lunex
    // world-space coordinates align with the visible viewport.
    #[allow(clippy::cast_possible_truncation)]
    let root_transform = Transform::from_xyz(SCREEN_WIDTH_PX as f32 / 2.0, SCREEN_HEIGHT_PX as f32 / 2.0, 0.0);

    commands
        .spawn((GameItem, HudRoot, UiLayoutRoot::new_2d(), UiFetchFromCamera::<0>, root_transform))
        .with_children(|root| {
            // Bottom HUD bar: occupies bottom 12% of screen.
            // Only the root carries GameItem; children are despawned
            // recursively when the root is removed by despawn_level.
            //
            // UiDepth must stay low: bevy_lunex sets accumulated depth as local z,
            // and Bevy's transform propagation adds parent z + child z. The
            // default orthographic camera has far=1000, so total global z must
            // stay well below that. Depth 10 renders above game sprites (z ≈ 0)
            // while keeping grandchildren (10 → 11 → 12) safely in range.
            root.spawn((HudBar, UiLayout::boundary().pos1(Rl((0.0, 88.0))).pos2(Rl(100.0)).pack(), UiDepth::Add(10.0)))
                .with_children(|bar| {
                    // Glow overlays — slightly oversized, semi-transparent panel sprites
                    // behind the main panels, pulsing slowly for a breathing border glow.
                    bar.spawn((
                        HudGlow { color: PANEL_BORDER_GLOW, base_alpha: 0.12, amplitude: 0.06, speed: 1.2, offset: 0.0 },
                        UiLayout::boundary().pos1(Rl((0.0, 1.0))).pos2(Rl((60.5, 99.0))).pack(),
                        UiDepth::Add(-0.5),
                        Sprite {
                            image: player_panel_image.clone(),
                            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                                border: BorderRect::all(PANEL_BORDER_PX),
                                ..Default::default()
                            }),
                            color: Color::srgba(0.30, 0.70, 1.00, 0.12),
                            ..Default::default()
                        },
                    ));

                    bar.spawn((
                        HudGlow { color: Color::srgba(0.25, 0.65, 0.85, 1.0), base_alpha: 0.10, amplitude: 0.05, speed: 1.2, offset: std::f32::consts::PI },
                        UiLayout::boundary().pos1(Rl((62.5, 1.0))).pos2(Rl((100.0, 99.0))).pack(),
                        UiDepth::Add(-0.5),
                        Sprite {
                            image: observer_panel_image.clone(),
                            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                                border: BorderRect::all(PANEL_BORDER_PX),
                                ..Default::default()
                            }),
                            color: Color::srgba(0.25, 0.65, 0.85, 0.10),
                            ..Default::default()
                        },
                    ));

                    // Left panel — player stats (left 60%, with margin).
                    bar.spawn((PlayerPanel, UiLayout::boundary().pos1(Rl((1.0, 4.0))).pos2(Rl((59.0, 96.0))).pack(), player_panel_sprite()))
                        .with_children(|panel| {
                            spawn_player_labels(panel, &font, &display_font);
                        });

                    // Right panel — observer clock (right 35%, with margin).
                    bar.spawn((ObserverPanel, UiLayout::boundary().pos1(Rl((64.0, 4.0))).pos2(Rl((99.0, 96.0))).pack(), observer_panel_sprite()))
                        .with_children(|panel| {
                            spawn_observer_labels(panel, &font, &display_font);
                        });
                });
        });
}

/// Spawns labeled readouts for the player stats panel with visual hierarchy.
///
/// Layout: two-column grid with section headers (display font) and grouped values.
/// Left column: TIME (`t_p`) and VELOCITY (v). Right column: GAMMA (`γ_v`, `γ_g`).
/// Decorative elements: bullet indicators, gauge dots, corner accents, sub-dividers, vertical/bottom dividers.
#[allow(clippy::too_many_lines)]
fn spawn_player_labels(panel: &mut ChildSpawnerCommands, font: &Handle<Font>, display_font: &Handle<Font>) {
    let text_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    let section_font = TextFont {
        font: display_font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    // Helper: small bullet indicator (monospace font) placed before section labels.
    let bullet = |x_pct: f32, y_pct: f32| {
        (
            UiLayout::window().pos(Rl((x_pct, y_pct))).anchor(Anchor::CENTER_LEFT).pack(),
            UiTextSize::from(Rh(9.0)),
            Text2d::new("▸"),
            text_font.clone(),
            UiColor::from(ACCENT_COLOR),
        )
    };

    // Panel header in display font.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 3.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(14.0)),
        Text2d::new("FLIGHT DATA"),
        TextFont {
            font: display_font.clone(),
            font_size: 64.0,
            ..Default::default()
        },
        UiColor::from(HEADER_COLOR),
    ));

    // Corner accent — top-right flourish.
    panel.spawn((
        UiLayout::window().pos(Rl((93.0, 3.0))).anchor(Anchor::CENTER_RIGHT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◇"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Horizontal divider under header.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 16.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("──────────────────────────────────"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Bullet indicators before section labels.
    panel.spawn(bullet(2.0, 26.0));
    panel.spawn(bullet(2.0, 60.0));
    panel.spawn(bullet(52.0, 26.0));

    // Left column — section label: TIME.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 26.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(11.0)),
        Text2d::new("TIME"),
        section_font.clone(),
        UiColor::from(LABEL_COLOR),
    ));

    // Left column — section label: VELOCITY.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 60.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(11.0)),
        Text2d::new("VELOCITY"),
        section_font.clone(),
        UiColor::from(LABEL_COLOR),
    ));

    // Right column — section label: GAMMA.
    panel.spawn((
        UiLayout::window().pos(Rl((55.0, 26.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(11.0)),
        Text2d::new("GAMMA"),
        section_font,
        UiColor::from(LABEL_COLOR),
    ));

    // Thin vertical divider between left and right columns.
    panel.spawn((
        UiLayout::window().pos(Rl((50.0, 24.0))).anchor(Anchor::TOP_CENTER).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("│\n│\n│\n│\n│"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Sub-divider between grouped gamma readouts.
    panel.spawn((
        UiLayout::window().pos(Rl((55.0, 55.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("── ── ── ── ──"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Bottom accent divider line.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 92.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("──────────────────────────────────"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Corner accent: bottom-left flourish (mirrors top-right).
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 97.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◇"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Gauge dot indicators beside value readouts.
    panel.spawn((
        UiLayout::window().pos(Rl((2.0, 42.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));
    panel.spawn((
        UiLayout::window().pos(Rl((2.0, 76.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));
    panel.spawn((
        UiLayout::window().pos(Rl((52.0, 42.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));
    panel.spawn((
        UiLayout::window().pos(Rl((52.0, 66.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Value readouts with marker components for targeted text updates.
    let value = |x_pct: f32, y_pct: f32, label: &str| {
        (
            UiLayout::window().pos(Rl((x_pct, y_pct))).anchor(Anchor::CENTER_LEFT).pack(),
            UiTextSize::from(Rh(22.0)),
            Text2d::new(label),
            text_font.clone(),
            UiColor::from(TEXT_COLOR),
        )
    };

    // Left column values.
    panel.spawn((value(5.0, 42.0, "t_p = 0.00"), HudPlayerTime, HudFlash::new()));
    panel.spawn((value(5.0, 76.0, "v = 0.00c"), HudVelocityFraction, HudFlash::new()));

    // Right column values (gamma grouped together).
    panel.spawn((value(55.0, 42.0, "γ_v = 1.00"), HudVelocityGamma, HudFlash::new()));
    panel.spawn((value(55.0, 66.0, "γ_g = 1.00"), HudGravGamma, HudFlash::new()));
}

/// Spawns labels for the observer clock panel with visual hierarchy.
///
/// Layout: two-column grid with section headers. Left: TIME (`t_o`). Right: RATE (r).
/// Decorative elements: bullet indicators, gauge dots, corner accents, and bottom divider.
#[allow(clippy::too_many_lines)]
fn spawn_observer_labels(panel: &mut ChildSpawnerCommands, font: &Handle<Font>, display_font: &Handle<Font>) {
    let text_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    let section_font = TextFont {
        font: display_font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    // Helper: small bullet indicator (monospace font) placed before section labels.
    let bullet = |x_pct: f32, y_pct: f32| {
        (
            UiLayout::window().pos(Rl((x_pct, y_pct))).anchor(Anchor::CENTER_LEFT).pack(),
            UiTextSize::from(Rh(9.0)),
            Text2d::new("▸"),
            text_font.clone(),
            UiColor::from(ACCENT_COLOR),
        )
    };

    // Panel header in display font.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 5.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(14.0)),
        Text2d::new("OBSERVER"),
        TextFont {
            font: display_font.clone(),
            font_size: 64.0,
            ..Default::default()
        },
        UiColor::from(HEADER_COLOR),
    ));

    // Corner accent — top-right flourish.
    panel.spawn((
        UiLayout::window().pos(Rl((93.0, 5.0))).anchor(Anchor::CENTER_RIGHT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◇"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Horizontal divider under header.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 20.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("──────────────────────────────────"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Bullet indicators before section labels.
    panel.spawn(bullet(2.0, 32.0));
    panel.spawn(bullet(52.0, 32.0));

    // Section labels.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 32.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(11.0)),
        Text2d::new("TIME"),
        section_font.clone(),
        UiColor::from(LABEL_COLOR),
    ));

    panel.spawn((
        UiLayout::window().pos(Rl((55.0, 32.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(11.0)),
        Text2d::new("RATE"),
        section_font,
        UiColor::from(LABEL_COLOR),
    ));

    // Bottom accent divider line.
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 85.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(8.0)),
        Text2d::new("──────────────────────────────────"),
        text_font.clone(),
        UiColor::from(DIVIDER_COLOR),
    ));

    // Corner accent: bottom-left flourish (mirrors top-right).
    panel.spawn((
        UiLayout::window().pos(Rl((5.0, 92.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◇"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Gauge dot indicators beside value readouts.
    panel.spawn((
        UiLayout::window().pos(Rl((2.0, 58.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));
    panel.spawn((
        UiLayout::window().pos(Rl((52.0, 58.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(9.0)),
        Text2d::new("◦"),
        text_font.clone(),
        UiColor::from(ACCENT_COLOR),
    ));

    // Value readouts.
    panel.spawn((
        HudObserverTime,
        HudFlash::new(),
        UiLayout::window().pos(Rl((5.0, 58.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(22.0)),
        Text2d::new("t_o = 0.00"),
        text_font.clone(),
        UiColor::from(TEXT_COLOR),
    ));

    panel.spawn((
        HudSimRate,
        HudFlash::new(),
        UiLayout::window().pos(Rl((55.0, 58.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(22.0)),
        Text2d::new("r = 1.00×"),
        text_font,
        UiColor::from(TEXT_COLOR),
    ));
}

/// Updates the individual player stat labels in the HUD with live data.
///
/// Reads `Clock`, `VelocityGamma`, `GravitationalGamma` from the `PlayerHud`
/// entity, and `Velocity` from the `Player` entity, then writes formatted
/// values into the four labeled `Text2d` components.
#[allow(clippy::type_complexity)]
pub fn player_hud_text_update(
    data_query: Query<(&Clock, &VelocityGamma, &GravitationalGamma), With<PlayerHud>>,
    velocity_query: Query<&Velocity, With<Player>>,
    mut tp_query: Query<(&mut Text2d, &mut UiColor), (With<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudGravGamma>, Without<HudVelocityFraction>)>,
    mut vel_gamma_query: Query<(&mut Text2d, &mut UiColor), (With<HudVelocityGamma>, Without<HudPlayerTime>, Without<HudGravGamma>, Without<HudVelocityFraction>)>,
    mut grav_gamma_query: Query<(&mut Text2d, &mut UiColor), (With<HudGravGamma>, Without<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudVelocityFraction>)>,
    mut vf_query: Query<(&mut Text2d, &mut UiColor), (With<HudVelocityFraction>, Without<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudGravGamma>)>,
) {
    let Ok((clock, velocity_gamma, gravitational_gamma)) = data_query.single() else { return };
    let Ok(velocity) = velocity_query.single() else { return };

    let days = clock.value.value / 24.0 / 3600.0;

    if let Ok((mut text, mut color)) = tp_query.single_mut() {
        **text = format!("t_p = {days:2.2}");
        *color = TEXT_COLOR.into();
    }

    if let Ok((mut text, mut color)) = vel_gamma_query.single_mut() {
        **text = format!("γ_v = {:2.2}", velocity_gamma.value);
        *color = gamma_to_hud_color(velocity_gamma.value).into();
    }

    if let Ok((mut text, mut color)) = grav_gamma_query.single_mut() {
        **text = format!("γ_g = {:2.2}", gravitational_gamma.value);
        *color = gamma_to_hud_color(gravitational_gamma.value).into();
    }

    if let Ok((mut text, mut color)) = vf_query.single_mut() {
        **text = format_velocity_fraction(velocity.scalar(), *C);
        *color = gamma_to_hud_color(velocity_gamma.value).into();
    }
}

/// Updates the observer time label in the HUD with live data.
///
/// Reads `Clock` from the `Observer` entity, then writes the formatted
/// value into the `HudObserverTime` `Text2d` component.
pub fn observer_hud_text_update(data_query: Query<&Clock, With<Observer>>, mut to_query: Query<(&mut Text2d, &mut UiColor), With<HudObserverTime>>) {
    let Ok(clock) = data_query.single() else { return };
    let Ok((mut text, mut color)) = to_query.single_mut() else { return };

    **text = format_observer_time(clock.value.value);
    *color = TEXT_COLOR.into();
}

/// Updates the simulation rate label in the HUD.
pub fn sim_rate_hud_update(sim_rate: Res<SimRate>, mut query: Query<(&mut Text2d, &mut UiColor), With<HudSimRate>>) {
    let Ok((mut text, mut color)) = query.single_mut() else { return };

    **text = format!("r = {:.2}×", sim_rate.0);
    *color = TEXT_COLOR.into();
}

/// Decays brightness flashes on HUD value readouts.
///
/// Runs after all HUD text-update systems. When a readout text
/// changes, the timer resets and a brightness boost decays over
/// `FLASH_DURATION_SECS`.
pub fn hud_flash_system(time: Res<Time>, mut query: Query<(&Text2d, &mut HudFlash, &mut UiColor)>) {
    for (text, mut flash, mut color) in &mut query {
        if text.as_str() != flash.prev_text {
            flash.prev_text.clone_from(text);
            flash.timer.reset();
        }

        flash.timer.tick(time.delta());

        if !flash.timer.is_finished() {
            let t = 1.0 - flash.timer.fraction();
            let boost = t * FLASH_BOOST;
            let base = color.to_srgba();
            *color = Color::srgba(
                (base.red + boost).min(1.0),
                (base.green + boost).min(1.0),
                (base.blue + boost).min(1.0),
                base.alpha,
            )
            .into();
        }
    }
}

/// Pulses panel glow overlay alpha for a subtle breathing border effect.
pub fn hud_glow_pulse_system(time: Res<Time>, mut query: Query<(&HudGlow, &mut Sprite)>) {
    for (glow, mut sprite) in &mut query {
        let t = time.elapsed_secs() * glow.speed + glow.offset;
        let c = glow.color.to_srgba();
        let alpha = (glow.base_alpha + glow.amplitude * t.sin()).clamp(0.0, 1.0);
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}