use crate::{game::levels::CurrentLevel, shared::state::AppState};
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), spawn_menu)
            .add_systems(OnExit(AppState::Menu), despawn_menu)
            .add_systems(Update, menu_button_interaction.run_if(in_state(AppState::Menu)));
    }
}

// Marker components.

#[derive(Component)]
struct MenuScreen;

#[derive(Component)]
struct LevelButton(CurrentLevel);

// Systems.

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/HackNerdFontMono-Regular.ttf");

    commands
        .spawn((
            MenuScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            // Title.
            parent.spawn((
                Text::new("relativity"),
                TextFont {
                    font: font.clone(),
                    font_size: 64.0,
                    ..Default::default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..Default::default()
                },
            ));

            // Level buttons.
            for &level in CurrentLevel::all() {
                parent
                    .spawn((
                        LevelButton(level),
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
                            Text::new(level.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: 32.0,
                                ..Default::default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
fn menu_button_interaction(
    interaction_query: Query<(&Interaction, &LevelButton), (Changed<Interaction>, With<Button>)>,
    mut current_level: ResMut<CurrentLevel>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, level_button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            *current_level = level_button.0;
            app_state.set(AppState::InGame);
        }
    }
}
