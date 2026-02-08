use crate::{
    game::{
        levels::CurrentLevel,
        shared::types::{NextLevelButton, SuccessOverlay},
    },
    shared::state::{AppState, GameState},
};
use bevy::prelude::*;

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
/// If there is a next level, advance `CurrentLevel` and transition through Menu
/// to trigger level despawn/respawn. If no next level, return to Menu.
#[allow(clippy::type_complexity)]
pub fn success_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NextLevelButton>)>,
    mut current_level: ResMut<CurrentLevel>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(next) = current_level.next() {
                *current_level = next;
            }

            // Transition to Menu to trigger OnExit(InGame) despawn.
            // From Menu, the player selects the next level (or it auto-starts if we add that later).
            app_state.set(AppState::Menu);
            game_state.set(GameState::Paused);
        }
    }
}
