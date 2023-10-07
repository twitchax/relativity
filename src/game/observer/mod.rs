use bevy::prelude::*;

use super::shared::{constants::DAYS_PER_SECOND_UOM, types::{Clock, GameItem}};

#[derive(Component, Default)]
pub struct Observer;

#[derive(Bundle, Default)]
pub struct ObserverClockBundle {
    pub item: GameItem,
    pub observer: Observer,
    pub clock: Clock,
    pub clock_text: TextBundle,
}

pub fn spawn_observer_clock(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let clock_text = TextBundle::from_section(
        "t_o = 00.00",
        TextStyle {
            font_size: 40.0,
            font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
            ..Default::default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        right: Val::Px(10.0),
        ..Default::default()
    });

    commands.spawn(ObserverClockBundle { clock_text, ..Default::default() });
}

// Clock systems.

pub fn observer_clock_update(mut query: Query<&mut Clock, With<Observer>>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * time.delta_seconds() as f64;

    let mut clock = query.single_mut();

    clock.value += time_elapsed;
}

pub fn observer_clock_text_update(mut query: Query<(&mut Text, &Clock), With<Observer>>) {
    let (mut text, clock) = query.single_mut();

    let days = clock.value.value / 24.0 / 3600.0;

    text.sections[0].value = format!("t_o = {:2.2}", days);
}
