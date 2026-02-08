// E2E headless test: verifies that collision between player and destination
// triggers GameState::Finished.
//
// Uses MinimalPlugins to provide enough infrastructure for the collision_check
// system without a GPU or window.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

use bevy::{prelude::*, state::app::StatesPlugin};
use relativity::{
    game::{
        destination::Destination,
        object::Planet,
        player::shared::Player,
        shared::{
            systems::collision_check,
            types::{Position, Radius},
        },
    },
    shared::state::GameState,
};
use uom::si::{f64::Length as UomLength, length::kilometer};

fn km(v: f64) -> UomLength {
    UomLength::new::<kilometer>(v)
}

/// Build a headless app with GameState and the collision_check system.
fn build_collision_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins).add_plugins(StatesPlugin).init_state::<GameState>().add_systems(Update, collision_check);

    app
}

/// Spawn a player entity with the given position and radius.
fn spawn_player(world: &mut World, pos_x: f64, pos_y: f64, radius: f64) -> Entity {
    world.spawn((Player, Position { x: km(pos_x), y: km(pos_y) }, Radius { value: km(radius) })).id()
}

/// Spawn a destination entity with the given position and radius.
fn spawn_destination(world: &mut World, pos_x: f64, pos_y: f64, radius: f64) -> Entity {
    world.spawn((Destination, Position { x: km(pos_x), y: km(pos_y) }, Radius { value: km(radius) })).id()
}

/// Spawn a planet entity with the given position and radius.
fn spawn_planet(world: &mut World, pos_x: f64, pos_y: f64, radius: f64) -> Entity {
    world.spawn((Planet, Position { x: km(pos_x), y: km(pos_y) }, Radius { value: km(radius) })).id()
}

// ---------- Collision triggers Finished ----------

#[test]
fn collision_with_destination_transitions_to_finished() {
    let mut app = build_collision_test_app();

    // Set initial state to Running (collision_check only runs during Running in-game,
    // but here we add the system unconditionally so we can test it directly).
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
    app.update(); // Apply the state transition.

    // Spawn player and destination at the same position with overlapping radii.
    spawn_player(app.world_mut(), 0.0, 0.0, 100.0);
    spawn_destination(app.world_mut(), 0.0, 0.0, 100.0);

    app.update(); // Run collision_check.

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Running, "state should still be Running this frame (NextState queued)");

    app.update(); // Apply the queued NextState.

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Finished, "collision with destination should transition to Finished");
}

#[test]
fn no_collision_when_far_apart() {
    let mut app = build_collision_test_app();

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
    app.update();

    // Player and destination far apart — no collision.
    spawn_player(app.world_mut(), 0.0, 0.0, 100.0);
    spawn_destination(app.world_mut(), 1_000_000.0, 1_000_000.0, 100.0);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Running, "no collision should leave state as Running");
}

#[test]
fn collision_with_planet_transitions_to_failed() {
    let mut app = build_collision_test_app();

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
    app.update();

    // Player overlaps with a planet — should transition to Failed.
    spawn_player(app.world_mut(), 0.0, 0.0, 100.0);
    spawn_destination(app.world_mut(), 1_000_000.0, 1_000_000.0, 100.0); // Far-away destination.
    spawn_planet(app.world_mut(), 0.0, 0.0, 100.0);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Failed, "collision with planet should transition to Failed");
}

#[test]
fn destination_collision_checked_before_planet() {
    let mut app = build_collision_test_app();

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
    app.update();

    // Player overlaps with both destination and planet at origin.
    // The collision_check system checks destination first, then planets.
    // Since NextState uses set() (last write wins), planet collision (Failed) overwrites destination (Finished).
    spawn_player(app.world_mut(), 0.0, 0.0, 100.0);
    spawn_destination(app.world_mut(), 0.0, 0.0, 100.0);
    spawn_planet(app.world_mut(), 0.0, 0.0, 100.0);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    // Planet collision sets Failed last, overwriting the Finished from destination.
    assert_eq!(*state.get(), GameState::Failed, "planet collision should overwrite destination collision (last-write-wins)");
}

#[test]
fn boundary_collision_touching_edges() {
    let mut app = build_collision_test_app();

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
    app.update();

    // Player and destination exactly touching (distance == sum of radii).
    // has_collided uses <=, so this should trigger.
    spawn_player(app.world_mut(), 0.0, 0.0, 50.0);
    spawn_destination(app.world_mut(), 100.0, 0.0, 50.0);

    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Finished, "touching edges (distance == r1 + r2) should trigger collision");
}
