use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::game::shared::types::GameItem;
use crate::shared::state::AppState;

// Paths.

/// 9-slice panel background sprite (dark with cyan border glow).
const PANEL_SPRITE: &str = "sprites/hud/panel.png";
/// Border pixel width in the panel sprite (must match the asset).
const PANEL_BORDER_PX: f32 = 4.0;

// Colors.

/// Soft white text color.
const TEXT_COLOR: Color = Color::srgba(0.9, 0.95, 1.0, 1.0);

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

/// Plugin that spawns the HUD layout root.
///
/// Requires `UiLunexPlugins` to be registered at the app level
/// (alongside `DefaultPlugins`) so that picking/cursor systems
/// have the resources they need.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud_root).add_systems(OnExit(AppState::InGame), despawn_hud_root);
    }
}

/// Spawns the `bevy_lunex` layout root and the bottom-anchored HUD bar
/// with player stats (left) and observer clock (right) panels.
fn spawn_hud_root(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/HackNerdFontMono-Regular.ttf");
    let panel_image: Handle<Image> = asset_server.load(PANEL_SPRITE);

    let panel_sprite = || Sprite {
        image: panel_image.clone(),
        image_mode: SpriteImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(PANEL_BORDER_PX),
            ..Default::default()
        }),
        ..Default::default()
    };

    commands.spawn((GameItem, HudRoot, UiLayoutRoot::new_2d(), UiFetchFromCamera::<0>)).with_children(|root| {
        // Bottom HUD bar: occupies bottom 12% of screen.
        root.spawn((GameItem, HudBar, UiLayout::boundary().pos1(Rl((0.0, 88.0))).pos2(Rl(100.0)).pack(), UiDepth::Add(900.0)))
            .with_children(|bar| {
                // Left panel — player stats (left 60%, with margin).
                bar.spawn((GameItem, PlayerPanel, UiLayout::boundary().pos1(Rl((1.0, 4.0))).pos2(Rl((59.0, 96.0))).pack(), panel_sprite()))
                    .with_children(|panel| {
                        spawn_player_labels(panel, &font);
                    });

                // Right panel — observer clock (right 35%, with margin).
                bar.spawn((GameItem, ObserverPanel, UiLayout::boundary().pos1(Rl((64.0, 4.0))).pos2(Rl((99.0, 96.0))).pack(), panel_sprite()))
                    .with_children(|panel| {
                        spawn_observer_labels(panel, &font);
                    });
            });
    });
}

/// Spawns placeholder labels for the player stats panel.
fn spawn_player_labels(panel: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let text_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    let labels = [("t_p = 0.00", 15.0), ("γ_v = 1.00", 38.0), ("γ_g = 1.00", 61.0), ("v = 0.00c", 84.0)];

    for (label, y_pct) in labels {
        panel.spawn((
            GameItem,
            UiLayout::window().pos(Rl((5.0, y_pct))).anchor(Anchor::CENTER_LEFT).pack(),
            UiTextSize::from(Rh(22.0)),
            Text2d::new(label),
            text_font.clone(),
            UiColor::from(TEXT_COLOR),
        ));
    }
}

/// Spawns placeholder label for the observer clock panel.
fn spawn_observer_labels(panel: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let text_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    panel.spawn((
        GameItem,
        UiLayout::window().pos(Rl((5.0, 50.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(22.0)),
        Text2d::new("t_o = 0.00"),
        text_font,
        UiColor::from(TEXT_COLOR),
    ));
}

/// Despawns the HUD root entity on state exit.
///
/// The `GameItem`-based despawn in `levels/mod.rs` handles this too,
/// but having an explicit despawn avoids ordering surprises.
fn despawn_hud_root(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
