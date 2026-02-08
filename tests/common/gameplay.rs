#![allow(dead_code)]

//! Shared test utilities for E2E gameplay tests.
//!
//! Provides helpers to build a headless Bevy app with the full `GamePlugin`,
//! transition state, launch the player, and run the physics loop.
//!
//! Why we don't emulate a mouse click:
//! Bevy's launch systems read `ButtonInput<MouseButton>` and cursor
//! position from `PrimaryWindow`.  In a headless test (no WinitPlugin), the
//! window's `cursor_position()` always returns `None`, so the systems
//! early-return.  We bypass input and directly set the player's Velocity,
//! which is what the launch systems ultimately compute from the click.  The
//! pure functions are already unit-tested separately.

use std::time::Duration;

use bevy::{input::InputPlugin, prelude::*, state::app::StatesPlugin, text::TextPlugin, time::TimeUpdateStrategy};
use bevy_trauma_shake::TraumaPlugin;
use relativity::{
    game::{
        player::shared::Player,
        shared::types::{Clock, Position, Velocity},
        GamePlugin,
    },
    shared::state::{AppState, GameState},
};
use uom::si::{f64::Velocity as UomVelocity, time::second, velocity::kilometer_per_second};

/// Build a headless Bevy app exactly as the real game does — `GamePlugin` with
/// default resources — plus deterministic time for reproducibility.
///
/// No level or game-state configuration is done here; the app boots with
/// whatever defaults the game defines.  If a level layout changes, the
/// hardcoded trajectories in the tests will (intentionally) break.
pub fn build_gameplay_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(TransformPlugin)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ImagePlugin::default())
        .add_plugins(TextPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(bevy::gizmos::GizmoPlugin)
        .add_plugins(TraumaPlugin)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<relativity::game::levels::CurrentLevel>()
        .init_state::<AppState>()
        .add_plugins(GamePlugin);

    app
}

/// Transition the app into `AppState::InGame`, triggering `spawn_level`.
pub fn enter_game(app: &mut App) {
    app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);
    app.update(); // Processes state transition + OnEnter(InGame) = spawn_level.
}

/// Find the player *sprite* entity (has `Player` + `Position`, but NOT `Clock`).
///
/// The player *clock* entity also carries `Player`, so we exclude it via
/// `Without<Clock>`.
pub fn find_player_sprite(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, (With<Player>, With<Position>, Without<Clock>)>()
        .single(app.world())
        .expect("expected exactly one Player sprite entity with Position and no Clock")
}

/// Find the player *clock* entity (has `Player` + `Clock`).
pub fn find_player_clock(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, (With<Player>, With<Clock>)>()
        .single(app.world())
        .expect("expected exactly one Player clock entity with Clock")
}

/// Read the current `GameState`.
pub fn current_game_state(app: &App) -> GameState {
    app.world().resource::<State<GameState>>().get().clone()
}

/// Set the player sprite's launch velocity (km/s) without changing position.
///
/// The player stays at its default spawn location; only velocity is set.
/// This mimics what the launch systems do after computing velocity from input.
pub fn launch_player(app: &mut App, player: Entity, vel_kms: (f64, f64)) {
    let mut vel = app.world_mut().get_mut::<Velocity>(player).unwrap();
    vel.x = UomVelocity::new::<kilometer_per_second>(vel_kms.0);
    vel.y = UomVelocity::new::<kilometer_per_second>(vel_kms.1);
}

/// Transition `GameState` to `Running` so physics and collision systems execute.
pub fn start_running(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Running);
}

/// Read the player clock value in days (matches the in-game display unit).
pub fn read_player_clock_days(app: &App, clock_entity: Entity) -> f64 {
    let seconds = app.world().get::<Clock>(clock_entity).unwrap().value.get::<second>();
    seconds / 24.0 / 3600.0
}

/// Run up to `max_frames` updates, returning the final `GameState`.
///
/// Stops early if the state leaves `Running` (either `Finished`, `Failed`, or `Paused`).
pub fn run_until_resolved(app: &mut App, max_frames: usize) -> GameState {
    for _ in 0..max_frames {
        app.update();

        let state = current_game_state(app);
        if state != GameState::Running {
            return state;
        }
    }

    GameState::Running // Budget exhausted without resolution.
}
