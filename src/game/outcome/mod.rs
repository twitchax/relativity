use crate::{
    game::{
        levels::CurrentLevel,
        shared::types::{FailureOverlay, FailureTimer, NextLevelButton, PendingNextLevel, SuccessOverlay},
    },
    shared::state::{AppState, GameState},
};
use bevy::prelude::*;
use bevy_trauma_shake::Shake;

// Systems.

/// Spawn the success overlay when entering `GameState::Finished`.
pub fn spawn_success_overlay(mut commands: Commands, asset_server: Res<AssetServer>, current_level: Res<CurrentLevel>) {
    let font = asset_server.load("fonts/HackNerdFontMono-Regular.ttf");
    let has_next = current_level.next().is_some();

    commands
        .spawn((
            SuccessOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(24.0),
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // SUCCESS text.
            parent.spawn((
                Text::new("SUCCESS"),
                TextFont {
                    font: font.clone(),
                    font_size: 72.0,
                    ..Default::default()
                },
                TextColor(Color::srgba(0.2, 1.0, 0.2, 1.0)),
            ));

            // Next Level / Return to Menu button.
            let button_text = if has_next { "Next Level" } else { "Menu" };

            parent
                .spawn((
                    NextLevelButton,
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..Default::default()
                    },
                    BorderColor::all(Color::WHITE),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(button_text),
                        TextFont {
                            font,
                            font_size: 32.0,
                            ..Default::default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Despawn the success overlay when leaving `GameState::Finished`.
pub fn despawn_success_overlay(mut commands: Commands, query: Query<Entity, With<SuccessOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle Next Level button interaction.
///
/// If there is a next level, advance `CurrentLevel`, insert `PendingNextLevel` marker,
/// and transition through Menu to trigger level despawn/respawn. The menu auto-advances
/// back to `InGame` when `PendingNextLevel` is present.
/// If no next level, return to Menu without the marker.
#[allow(clippy::type_complexity)]
pub fn success_button_interaction(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NextLevelButton>)>,
    mut current_level: ResMut<CurrentLevel>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(next) = current_level.next() {
                *current_level = next;
                commands.insert_resource(PendingNextLevel);
            }

            // Transition to Menu to trigger OnExit(InGame) despawn.
            // If PendingNextLevel is set, the menu will auto-advance to InGame.
            app_state.set(AppState::Menu);
            game_state.set(GameState::Paused);
        }
    }
}

// Failure overlay systems.

/// Apply camera shake trauma when entering `GameState::Failed`.
///
/// This runs as an `OnEnter` system so the shake starts immediately on collision,
/// concurrent with the failure overlay.
pub fn apply_collision_shake(mut shakes: Query<&mut Shake>) {
    for mut shake in &mut shakes {
        shake.add_trauma(0.4);
    }
}

/// Spawn the failure overlay and insert the auto-reset timer when entering `GameState::Failed`.
pub fn spawn_failure_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/HackNerdFontMono-Regular.ttf");

    commands.insert_resource(FailureTimer(Timer::from_seconds(1.5, TimerMode::Once)));

    commands
        .spawn((
            FailureOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("FAILURE"),
                TextFont {
                    font,
                    font_size: 72.0,
                    ..Default::default()
                },
                TextColor(Color::srgba(1.0, 0.2, 0.2, 1.0)),
            ));
        });
}

/// Despawn the failure overlay and remove the timer when leaving `GameState::Failed`.
pub fn despawn_failure_overlay(mut commands: Commands, query: Query<Entity, With<FailureOverlay>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<FailureTimer>();
}

/// Tick the failure timer and transition back to `GameState::Paused` when it finishes.
pub fn failure_auto_reset(time: Res<Time>, mut timer: ResMut<FailureTimer>, mut game_state: ResMut<NextState<GameState>>) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        game_state.set(GameState::Paused);
    }
}
