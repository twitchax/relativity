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
        ObserverClockBundle { clock_text, node, ..Default::default() },
        TextFont {
            font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
            font_size: 40.0,
            ..Default::default()
        },
    ));
}

// Pure functions.

/// Format an observer clock value (in seconds) as a display string showing days.
#[must_use]
pub(crate) fn format_observer_time(clock_value_seconds: f64) -> String {
    let days = clock_value_seconds / 24.0 / 3600.0;
    format!("t_o = {days:2.2}")
}

// Clock systems.

#[allow(clippy::needless_pass_by_value)]
pub fn observer_clock_update(mut query: Query<&mut Clock, With<Observer>>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * f64::from(time.delta_secs());

    let Ok(mut clock) = query.single_mut() else { return };

    clock.value += time_elapsed;
}

pub fn observer_clock_text_update(mut query: Query<(&mut Text, &Clock), With<Observer>>) {
    let Ok((mut text, clock)) = query.single_mut() else { return };

    // In Bevy 0.17, Text implements Deref<Target = String>, so we use **text to mutate the underlying String.
    **text = format_observer_time(clock.value.value);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn format_observer_time_zero_seconds() {
        assert_eq!(format_observer_time(0.0), "t_o = 0.00");
    }

    #[test]
    fn format_observer_time_one_day() {
        let one_day_seconds = 24.0 * 3600.0;
        assert_eq!(format_observer_time(one_day_seconds), "t_o = 1.00");
    }

    #[test]
    fn format_observer_time_half_day() {
        let half_day_seconds = 12.0 * 3600.0;
        assert_eq!(format_observer_time(half_day_seconds), "t_o = 0.50");
    }

    #[test]
    fn format_observer_time_multiple_days() {
        let ten_days_seconds = 10.0 * 24.0 * 3600.0;
        assert_eq!(format_observer_time(ten_days_seconds), "t_o = 10.00");
    }

    #[test]
    fn format_observer_time_fractional_day() {
        let seconds = 1.5 * 24.0 * 3600.0;
        assert_eq!(format_observer_time(seconds), "t_o = 1.50");
    }

    #[test]
    fn format_observer_time_small_value() {
        let seconds = 3600.0; // 1 hour = 1/24 of a day
        assert_eq!(format_observer_time(seconds), "t_o = 0.04");
    }

    #[test]
    fn format_observer_time_large_value() {
        let seconds = 365.0 * 24.0 * 3600.0;
        assert_eq!(format_observer_time(seconds), "t_o = 365.00");
    }

    #[test]
    fn format_observer_time_negative_value() {
        let seconds = -24.0 * 3600.0;
        assert_eq!(format_observer_time(seconds), "t_o = -1.00");
    }

    #[test]
    fn format_observer_time_prefix_present() {
        assert!(format_observer_time(0.0).starts_with("t_o = "));
    }
}
