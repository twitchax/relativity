// E2E headless test: verifies that pressing G toggles gravity grid visibility on/off.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, prelude::*};
use relativity::game::shared::types::GridVisible;

/// Press G via the grid_toggle system.
fn press_g_toggle(app: &mut App) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyG);

    app.world_mut().run_system_once(relativity::game::shared::systems::grid_toggle).expect("grid_toggle should run");

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyG);
}

#[test]
fn g_toggles_grid_visibility_off_then_on() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Grid should be visible by default.
    assert!(app.world().resource::<GridVisible>().0, "grid should default to visible");

    // Press G → grid should be hidden.
    press_g_toggle(&mut app);
    assert!(!app.world().resource::<GridVisible>().0, "grid should be hidden after first G press");

    // Press G again → grid should be visible again.
    press_g_toggle(&mut app);
    assert!(app.world().resource::<GridVisible>().0, "grid should be visible after second G press");
}

#[test]
fn g_toggle_works_during_running_state() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Transition to Running state.
    let player = common::find_player_sprite(&mut app);
    common::launch_player(&mut app, player, (50.0, 50.0));
    common::start_running(&mut app);
    app.update();

    assert!(app.world().resource::<GridVisible>().0, "grid should be visible initially");

    // Press G during Running → should toggle off.
    press_g_toggle(&mut app);
    assert!(!app.world().resource::<GridVisible>().0, "grid should be hidden during Running after G press");
}
