// E2E headless test: verifies that despawn_level removes all GameItem entities.
//
// Uses the same MinimalPlugins setup as e2e_level_spawning, then runs despawn_level
// and asserts no GameItem entities remain.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

use bevy::{ecs::system::RunSystemOnce, prelude::*, text::TextPlugin};
use relativity::game::{
    destination::Destination,
    levels::{despawn_level, spawn_level, CurrentLevel},
    object::Planet,
    observer::Observer,
    player::shared::Player,
    shared::types::GameItem,
};

/// Build a headless app suitable for testing level spawning and despawning.
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

/// Helper to count entities matching a filter.
fn count_entities<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    app.world_mut().query_filtered::<Entity, F>().iter(app.world()).count()
}

// ---------- Level 1: despawn ----------

#[test]
fn despawn_level1_removes_all_game_items() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    // Verify entities were spawned.
    let before = count_entities::<With<GameItem>>(&mut app);
    assert!(before > 0, "Level 1 should have spawned GameItem entities");

    // Run despawn_level.
    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    let after = count_entities::<With<GameItem>>(&mut app);
    assert_eq!(after, 0, "All GameItem entities should be removed after despawn_level");
}

#[test]
fn despawn_level1_removes_all_players() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    assert!(count_entities::<With<Player>>(&mut app) > 0, "Should have Player entities before despawn");

    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    assert_eq!(count_entities::<With<Player>>(&mut app), 0, "No Player entities should remain after despawn");
}

#[test]
fn despawn_level1_removes_all_planets() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    assert!(count_entities::<With<Planet>>(&mut app) > 0, "Should have Planet entities before despawn");

    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    assert_eq!(count_entities::<With<Planet>>(&mut app), 0, "No Planet entities should remain after despawn");
}

#[test]
fn despawn_level1_removes_destination() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    assert!(count_entities::<With<Destination>>(&mut app) > 0, "Should have Destination before despawn");

    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    assert_eq!(count_entities::<With<Destination>>(&mut app), 0, "No Destination should remain after despawn");
}

#[test]
fn despawn_level1_removes_observer() {
    let mut app = build_level_test_app(CurrentLevel::One);
    app.update();

    assert!(count_entities::<With<Observer>>(&mut app) > 0, "Should have Observer before despawn");

    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    assert_eq!(count_entities::<With<Observer>>(&mut app), 0, "No Observer should remain after despawn");
}

// ---------- TimeWarp: despawn ----------

#[test]
fn despawn_time_warp_removes_all_game_items() {
    let mut app = build_level_test_app(CurrentLevel::TimeWarp);
    app.update();

    let before = count_entities::<With<GameItem>>(&mut app);
    assert!(before > 0, "TimeWarp should have spawned GameItem entities");

    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    let after = count_entities::<With<GameItem>>(&mut app);
    assert_eq!(after, 0, "All GameItem entities should be removed after despawn_level (TimeWarp)");
}

// ---------- Edge case: despawn on empty world ----------

#[test]
fn despawn_on_empty_world_is_noop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();

    // No GameItem entities exist — despawn_level should be a no-op.
    app.world_mut().run_system_once(despawn_level).unwrap();
    app.update();

    let count = count_entities::<With<GameItem>>(&mut app);
    assert_eq!(count, 0, "Despawning an empty world should not panic and leave zero GameItem entities");
}
