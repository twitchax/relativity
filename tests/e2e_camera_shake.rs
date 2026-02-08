// E2E headless test: verifies that transitioning to GameState::Failed
// applies camera shake trauma via the apply_collision_shake system.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::prelude::*;
use bevy::reflect::Struct;
use bevy_trauma_shake::Shake;
use relativity::shared::state::GameState;

#[test]
fn camera_shake_trauma_applied_on_failed() {
    let mut app = common::build_gameplay_app();

    // Spawn a camera with Shake (in the real app, spawn_camera runs at Startup in main.rs,
    // but the gameplay test app only includes GamePlugin).
    let shake_entity = app.world_mut().spawn((Camera2d, Shake::default())).id();

    common::enter_game(&mut app);

    let shake = app.world().get::<Shake>(shake_entity).unwrap();
    let initial_trauma = read_trauma(shake);
    assert!(initial_trauma < f32::EPSILON, "expected initial trauma to be ~0, got {initial_trauma}");

    // Transition to Failed — this triggers OnEnter(Failed) which runs apply_collision_shake.
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Failed);
    app.update();

    assert_eq!(common::current_game_state(&app), GameState::Failed);

    // After entering Failed, the Shake component should have trauma > 0.
    let shake = app.world().get::<Shake>(shake_entity).unwrap();
    let trauma_after = read_trauma(shake);
    assert!(trauma_after > 0.3, "expected trauma ~0.4 after planet collision, got {trauma_after}");
}

/// Read the private `trauma` field from a `Shake` component via Bevy's Reflect API.
fn read_trauma(shake: &Shake) -> f32 {
    let field = shake.field("trauma").expect("Shake should have a 'trauma' field via Reflect");
    *field.try_downcast_ref::<f32>().expect("trauma field should be f32")
}
