use bevy::prelude::*;

use super::shared::{
    constants::DAYS_PER_SECOND_UOM,
    types::{Clock, GameItem},
};

#[derive(Component, Default)]
pub struct Observer;

#[derive(Bundle, Default)]
pub struct ObserverClockBundle {
    pub item: GameItem,
    pub observer: Observer,
    pub clock: Clock,
    pub clock_text: Text,
    pub node: Node,
}

pub fn spawn_observer_clock(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let clock_text = Text::new("t_o = 00.00");

    let node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        right: Val::Px(10.0),
        ..Default::default()
    };

    commands.spawn((
        ObserverClockBundle {
            clock_text,
            node,
            ..Default::default()
        },
        TextFont {
            font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
            font_size: 40.0,
            ..Default::default()
        },
    ));
}

// Clock systems.

pub fn observer_clock_update(mut query: Query<&mut Clock, With<Observer>>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * time.delta_secs() as f64;

    let Ok(mut clock) = query.single_mut() else { return };

    clock.value += time_elapsed;
}

pub fn observer_clock_text_update(mut query: Query<(&mut Text, &Clock), With<Observer>>) {
    let Ok((mut text, clock)) = query.single_mut() else { return };

    let days = clock.value.value / 24.0 / 3600.0;

    // In Bevy 0.17, Text implements Deref<Target = String>, so we use **text to mutate the underlying String.
    **text = format!("t_o = {:2.2}", days);
}
