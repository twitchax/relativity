// E2E test: verifies that the old bottom-center power bar UI (bevy_ui Node
// entities) is no longer spawned by the launch visual system.  The replacement
// radial arc uses Gizmos (immediate-mode), so no UI entities should appear.

#![allow(clippy::unwrap_used)]
#![allow(clippy::pedantic)]

mod common;

use bevy::{ecs::system::RunSystemOnce, ui::Node};
use relativity::game::{player::player_sprite::launch_visual_system, shared::types::LaunchState};

/// After running `launch_visual_system` in Launching state, no new bevy_ui
/// `Node` entities are created — confirming the old power bar UI has been removed.
/// (The new radial arc uses Gizmos only, no UI entities.)
#[test]
fn no_power_bar_ui_nodes_spawned_during_launching() {
    let mut app = common::build_gameplay_app();
    common::enter_game(&mut app);

    // Count pre-existing Node entities (e.g. HUD elements).
    let before = app.world_mut().query::<&Node>().iter(app.world()).count();

    // Enter Launching state (this is where the old power bar was spawned).
    let angle = std::f32::consts::FRAC_PI_4;
    let power = 0.8_f32;
    *app.world_mut().resource_mut::<LaunchState>() = LaunchState::Launching { angle, power };

    // Run the visual system that formerly spawned the PowerBarUi entity.
    app.world_mut().run_system_once(launch_visual_system).expect("launch_visual_system should run");
    app.update();

    // Verify no new Node entities were spawned (old power bar removed).
    let after = app.world_mut().query::<&Node>().iter(app.world()).count();
    assert_eq!(after, before, "launch_visual_system should not spawn UI Node entities; before={before}, after={after}");
}
