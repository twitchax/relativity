use bevy::prelude::*;

use crate::shared::state::{AppState, GameState};

/// Marker component for the persistent full-screen fade overlay.
#[derive(Component)]
pub struct FadeOverlay;

/// Direction of the current fade animation.
#[derive(Debug, Clone, PartialEq)]
pub enum FadeDirection {
    /// Fading to black (alpha 0 → 1). Holds the target `AppState` and `GameState` to apply once opaque.
    Out { next_app_state: AppState, next_game_state: GameState },
    /// Fading from black (alpha 1 → 0).
    In,
}

/// Resource tracking the current fade animation state.
///
/// When `None`, the overlay is fully transparent and no animation is running.
/// When `Some`, the fade system interpolates the overlay alpha each frame.
#[derive(Resource, Default)]
pub struct FadeState {
    pub active: Option<FadeDirection>,
    pub timer: Timer,
}

/// Duration of each fade direction (out or in) in seconds.
const FADE_DURATION_SECS: f32 = 0.3;

impl FadeState {
    /// Start a fade-out animation that will transition to the given states once opaque.
    pub fn start_fade_out(&mut self, next_app_state: AppState, next_game_state: GameState) {
        self.active = Some(FadeDirection::Out { next_app_state, next_game_state });
        self.timer = Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once);
    }

    /// Start a fade-in animation (from black to transparent).
    pub fn start_fade_in(&mut self) {
        self.active = Some(FadeDirection::In);
        self.timer = Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once);
    }
}

/// Spawn the persistent fade overlay at startup. Starts fully transparent.
pub fn spawn_fade_overlay(mut commands: Commands) {
    commands.spawn((
        FadeOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        // Render above other UI (success/failure overlays use GlobalZIndex(100)).
        GlobalZIndex(200),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        // Disable picking so the overlay never intercepts clicks on buttons below.
        Pickable::IGNORE,
    ));
}

/// Animate the fade overlay alpha and execute state transitions when a fade-out completes.
pub fn fade_update_system(
    time: Res<Time>,
    mut fade: ResMut<FadeState>,
    mut overlay_query: Query<&mut BackgroundColor, With<FadeOverlay>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Some(direction) = fade.active.clone() else {
        return;
    };

    fade.timer.tick(time.delta());
    let progress = fade.timer.fraction();

    let alpha = match &direction {
        FadeDirection::Out { .. } => progress,
        FadeDirection::In => 1.0 - progress,
    };

    for mut bg in &mut overlay_query {
        bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
    }

    if fade.timer.just_finished() {
        match direction {
            FadeDirection::Out { next_app_state, next_game_state } => {
                // Apply the deferred state transition now that the screen is black.
                app_state.set(next_app_state);
                game_state.set(next_game_state);

                // Immediately start the fade-in.
                fade.start_fade_in();
            }
            FadeDirection::In => {
                // Fade complete — ensure overlay is fully transparent and clear state.
                for mut bg in &mut overlay_query {
                    bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
                fade.active = None;
            }
        }
    }
}

/// Helper: returns `true` if a fade animation is currently in progress,
/// which callers can use to suppress input during transitions.
#[must_use]
pub fn is_fading(fade: &FadeState) -> bool {
    fade.active.is_some()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn fade_state_default_is_inactive() {
        let state = FadeState::default();
        assert!(state.active.is_none());
    }

    #[test]
    fn start_fade_out_sets_direction_and_timer() {
        let mut state = FadeState::default();
        state.start_fade_out(AppState::InGame, GameState::Paused);

        assert!(matches!(
            state.active,
            Some(FadeDirection::Out {
                next_app_state: AppState::InGame,
                next_game_state: GameState::Paused,
            })
        ));
        assert!(!state.timer.is_finished());
    }

    #[test]
    fn start_fade_in_sets_direction() {
        let mut state = FadeState::default();
        state.start_fade_in();

        assert!(matches!(state.active, Some(FadeDirection::In)));
        assert!(!state.timer.is_finished());
    }

    #[test]
    fn is_fading_returns_false_when_inactive() {
        let state = FadeState::default();
        assert!(!is_fading(&state));
    }

    #[test]
    fn is_fading_returns_true_when_fading_out() {
        let mut state = FadeState::default();
        state.start_fade_out(AppState::Menu, GameState::Paused);
        assert!(is_fading(&state));
    }

    #[test]
    fn is_fading_returns_true_when_fading_in() {
        let mut state = FadeState::default();
        state.start_fade_in();
        assert!(is_fading(&state));
    }

    #[test]
    fn fade_duration_is_positive() {
        const { assert!(FADE_DURATION_SECS > 0.0) };
    }
}
