use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::game::observer::{format_observer_time, Observer};
use crate::game::player::player_clock::format_velocity_fraction;
use crate::game::player::shared::Player;
use crate::game::shared::constants::C;
use crate::game::shared::types::{Clock, GameItem, GravitationalGamma, PlayerHud, Velocity, VelocityGamma};
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

/// Spawns labeled readouts for the player stats panel.
fn spawn_player_labels(panel: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    let text_font = TextFont {
        font: font.clone(),
        font_size: 64.0,
        ..Default::default()
    };

    // Each label gets its own marker component for targeted text updates.
    let base = |y_pct: f32, label: &str| {
        (
            GameItem,
            UiLayout::window().pos(Rl((5.0, y_pct))).anchor(Anchor::CENTER_LEFT).pack(),
            UiTextSize::from(Rh(22.0)),
            Text2d::new(label),
            text_font.clone(),
            UiColor::from(TEXT_COLOR),
        )
    };

    panel.spawn((base(15.0, "t_p = 0.00"), HudPlayerTime));
    panel.spawn((base(38.0, "γ_v = 1.00"), HudVelocityGamma));
    panel.spawn((base(61.0, "γ_g = 1.00"), HudGravGamma));
    panel.spawn((base(84.0, "v = 0.00c"), HudVelocityFraction));
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
        HudObserverTime,
        UiLayout::window().pos(Rl((5.0, 50.0))).anchor(Anchor::CENTER_LEFT).pack(),
        UiTextSize::from(Rh(22.0)),
        Text2d::new("t_o = 0.00"),
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
    mut tp_query: Query<&mut Text2d, (With<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudGravGamma>, Without<HudVelocityFraction>)>,
    mut vel_gamma_query: Query<&mut Text2d, (With<HudVelocityGamma>, Without<HudPlayerTime>, Without<HudGravGamma>, Without<HudVelocityFraction>)>,
    mut grav_gamma_query: Query<&mut Text2d, (With<HudGravGamma>, Without<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudVelocityFraction>)>,
    mut vf_query: Query<&mut Text2d, (With<HudVelocityFraction>, Without<HudPlayerTime>, Without<HudVelocityGamma>, Without<HudGravGamma>)>,
) {
    let Ok((clock, velocity_gamma, gravitational_gamma)) = data_query.single() else { return };
    let Ok(velocity) = velocity_query.single() else { return };

    let days = clock.value.value / 24.0 / 3600.0;

    if let Ok(mut text) = tp_query.single_mut() {
        **text = format!("t_p = {days:2.2}");
    }

    if let Ok(mut text) = vel_gamma_query.single_mut() {
        **text = format!("γ_v = {:2.2}", velocity_gamma.value);
    }

    if let Ok(mut text) = grav_gamma_query.single_mut() {
        **text = format!("γ_g = {:2.2}", gravitational_gamma.value);
    }

    if let Ok(mut text) = vf_query.single_mut() {
        **text = format_velocity_fraction(velocity.scalar(), *C);
    }
}

/// Updates the observer time label in the HUD with live data.
///
/// Reads `Clock` from the `Observer` entity, then writes the formatted
/// value into the `HudObserverTime` `Text2d` component.
pub fn observer_hud_text_update(data_query: Query<&Clock, With<Observer>>, mut to_query: Query<&mut Text2d, With<HudObserverTime>>) {
    let Ok(clock) = data_query.single() else { return };
    let Ok(mut text) = to_query.single_mut() else { return };

    **text = format_observer_time(clock.value.value);
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
