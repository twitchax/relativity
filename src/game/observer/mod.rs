use bevy::prelude::*;

use super::shared::{
    constants::DAYS_PER_SECOND_UOM,
    types::{Clock, GameItem},
};

#[derive(Component, Default)]
pub struct Observer;

/// Data-only bundle that holds the observer's clock.
///
/// The visual HUD is rendered by `bevy_lunex` in `src/game/hud/`; this bundle
/// only stores the authoritative data that those HUD systems read from.
#[derive(Bundle, Default)]
pub struct ObserverClockBundle {
    pub item: GameItem,
    pub observer: Observer,
    pub clock: Clock,
}

/// Spawns the data-only observer clock entity.
///
/// This entity holds the authoritative observer `Clock` value.
/// The visual HUD reads from this entity via the `Observer` marker
/// (see `src/game/hud/`).
pub fn spawn_observer_clock(commands: &mut Commands) {
    commands.spawn(ObserverClockBundle::default());
}

// Pure functions.

/// Format an observer clock value (in seconds) as a display string showing days.
#[must_use]
pub(crate) fn format_observer_time(clock_value_seconds: f64) -> String {
    let days = clock_value_seconds / 24.0 / 3600.0;
    format!("t_o = {days:2.2}")
}

// Clock systems.

pub fn observer_clock_update(mut query: Query<&mut Clock, With<Observer>>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * f64::from(time.delta_secs());

    let Ok(mut clock) = query.single_mut() else { return };

    clock.value += time_elapsed;
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
