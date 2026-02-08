use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::game::shared::types::GameItem;
use crate::shared::state::AppState;

/// Marker component for the HUD layout root entity.
#[derive(Component, Default)]
pub struct HudRoot;

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

/// Spawns the `bevy_lunex` layout root for the in-game HUD.
fn spawn_hud_root(mut commands: Commands) {
    commands.spawn((GameItem, HudRoot, UiLayoutRoot::new_2d(), UiFetchFromCamera::<0>));
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
