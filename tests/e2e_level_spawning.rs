// E2E headless test: verifies that level spawning creates the expected entities
// with the correct marker components.
//
// Uses MinimalPlugins + TransformPlugin + AssetPlugin to provide enough
// infrastructure for the level spawn functions without a GPU or window.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

use bevy::{prelude::*, text::TextPlugin};
use relativity::game::{
    destination::Destination,
    levels::{spawn_level, CurrentLevel},
    object::Planet,
    observer::Observer,
    player::shared::Player,
    shared::types::GameItem,
};

/// Build a headless app suitable for testing level spawning.
///
/// Uses `MinimalPlugins` with `TransformPlugin`, `AssetPlugin`, `ImagePlugin`,
/// and `TextPlugin` — the minimum set needed by the level spawn functions
/// which load sprite images and font assets.
fn build_level_test_app(level: CurrentLevel) -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ImagePlugin::default())
        .add_plugins(TextPlugin)
        .insert_resource(level)
        .add_systems(Startup, spawn_level);

    app
}

/// Helper to count entities matching a filter after one update cycle.
fn count_entities<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut().query_filtered::<Entity, F>().iter(app.world()).count()
}

// ---------- Level 1 ----------

#[test]
fn level1_spawns_correct_game_item_count() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    let count = count_entities::<With<GameItem>>(&mut app);
    // Level 1: player + player_clock + observer_clock + 3 planets + 1 destination = 7
    assert_eq!(count, 7, "Level 1 should spawn 7 GameItem entities");
}

#[test]
fn level1_spawns_one_player() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    let count = count_entities::<With<Player>>(&mut app);
    // Player sprite + player clock both have Player component
    assert_eq!(count, 2, "Level 1 should have 2 entities with Player component (sprite + clock)");
}

#[test]
fn level1_spawns_three_planets() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    let count = count_entities::<With<Planet>>(&mut app);
    assert_eq!(count, 3, "Level 1 should spawn 3 Planet entities (SUN, SUN2, EARTH)");
}

#[test]
fn level1_spawns_one_destination() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    let count = count_entities::<With<Destination>>(&mut app);
    assert_eq!(count, 1, "Level 1 should spawn 1 Destination entity");
}

#[test]
fn level1_spawns_one_observer() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    let count = count_entities::<With<Observer>>(&mut app);
    assert_eq!(count, 1, "Level 1 should spawn 1 Observer entity");
}

// ---------- TimeWarp ----------

#[test]
fn time_warp_spawns_correct_game_item_count() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let count = count_entities::<With<GameItem>>(&mut app);
    // TimeWarp: player + player_clock + observer_clock + 1 dynamic planet + 1 destination = 5
    assert_eq!(count, 5, "TimeWarp should spawn 5 GameItem entities");
}

#[test]
fn time_warp_spawns_one_player() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let count = count_entities::<With<Player>>(&mut app);
    assert_eq!(count, 2, "TimeWarp should have 2 entities with Player component (sprite + clock)");
}

#[test]
fn time_warp_spawns_one_planet() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let count = count_entities::<With<Planet>>(&mut app);
    assert_eq!(count, 1, "TimeWarp should spawn 1 Planet entity (dynamic gravity well)");
}

#[test]
fn time_warp_spawns_one_destination() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let count = count_entities::<With<Destination>>(&mut app);
    assert_eq!(count, 1, "TimeWarp should spawn 1 Destination entity");
}

#[test]
fn time_warp_spawns_one_observer() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let count = count_entities::<With<Observer>>(&mut app);
    assert_eq!(count, 1, "TimeWarp should spawn 1 Observer entity");
}
