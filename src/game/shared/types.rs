use bevy::prelude::*;
use uom::si::f64::{Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity};

#[derive(Component, Default)]
pub struct GameItem;

/// Marker for the player HUD text entity (clock, gamma, velocity display).
#[derive(Component, Default)]
pub struct PlayerHud;

/// Two-phase launch state machine.
///
/// - `Idle`: waiting for the player to click.
/// - `AimLocked`: click registered; direction line rendered via Gizmos.
/// - `Launching`: holding and dragging to set power; power bar UI visible.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub enum LaunchState {
    #[default]
    Idle,
    AimLocked {
        angle: f32,
    },
    Launching {
        angle: f32,
        power: f32,
    },
}

/// Marker for the power-bar UI overlay spawned during the Launching phase.
#[derive(Component)]
pub struct PowerBarUi;

/// Marker for the velocity readout text entity shown near the radial arc during launch.
#[derive(Component)]
pub struct VelocityReadout;

/// Marker for the success overlay spawned when the player reaches the destination.
#[derive(Component)]
pub struct SuccessOverlay;

/// Marker for the "Next Level" button inside the success overlay.
#[derive(Component)]
pub struct NextLevelButton;

/// Marker for the failure overlay spawned on planet collision.
#[derive(Component)]
pub struct FailureOverlay;

/// Timer resource that drives the auto-reset delay after failure.
#[derive(Resource)]
pub struct FailureTimer(pub Timer);

/// Marker resource inserted when the "Next Level" button is pressed to signal
/// that the menu should automatically advance to `InGame` without user interaction.
#[derive(Resource)]
pub struct PendingNextLevel;

/// Marker resource inserted when the failure timer finishes to signal
/// that the level should be despawned and respawned on the next `Paused` entry.
#[derive(Resource)]
pub struct PendingLevelReset;

/// Ring buffer of recent player positions with gamma-based colors for trajectory trail rendering.
///
/// Each frame during `Running`, the current screen position and a color derived from
/// `gamma_v * gamma_g` are pushed. Rendered via `Gizmos::linestrip_gradient_2d`.
#[derive(Component, Default)]
pub struct TrailBuffer {
    pub points: Vec<(Vec2, Color)>,
}

#[derive(Component, Default)]
pub struct PlanetSprite;

#[derive(Component, Default)]
pub struct RocketSprite;

#[derive(Component, Default)]
pub struct Position {
    pub x: UomLength,
    pub y: UomLength,
}

#[derive(Component, Default)]
pub struct Radius {
    pub value: UomLength,
}

#[derive(Component, Default)]
pub struct Mass {
    pub value: UomMass,
}

#[derive(Component, Default)]
pub struct Velocity {
    pub x: UomVelocity,
    pub y: UomVelocity,
}

impl Velocity {
    #[must_use]
    pub fn scalar(&self) -> UomVelocity {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Component, Default)]
pub struct Clock {
    pub value: UomTime,
}

#[derive(Component, Default)]
pub struct VelocityGamma {
    pub value: f64,
}

#[derive(Component, Default)]
pub struct GravitationalGamma {
    pub value: f64,
}

/// Simulation speed multiplier applied to all time-dependent systems.
///
/// Range: [0.25, 2.00], step size 0.25. Default 1.0.
#[derive(Resource)]
pub struct SimRate(pub f64);

impl Default for SimRate {
    fn default() -> Self {
        Self(1.0)
    }
}

impl SimRate {
    pub const MAX: f64 = 2.0;
    pub const MIN: f64 = 0.25;
    pub const STEP: f64 = 0.25;
}

/// Controls whether the gravity grid overlay is rendered.
///
/// Default `true`. Toggled by the G key.
#[derive(Resource)]
pub struct GridVisible(pub bool);

impl Default for GridVisible {
    fn default() -> Self {
        Self(true)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::velocity::meter_per_second;

    fn make_velocity(x: f64, y: f64) -> Velocity {
        Velocity {
            x: UomVelocity::new::<meter_per_second>(x),
            y: UomVelocity::new::<meter_per_second>(y),
        }
    }

    // --- Velocity::scalar ---

    #[test]
    fn scalar_zero_velocity_returns_zero() {
        let v = make_velocity(0.0, 0.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 0.0);
    }

    #[test]
    fn scalar_unit_x_velocity() {
        let v = make_velocity(1.0, 0.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 1.0);
    }

    #[test]
    fn scalar_unit_y_velocity() {
        let v = make_velocity(0.0, 1.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 1.0);
    }

    #[test]
    fn scalar_pythagorean_3_4_5() {
        let v = make_velocity(3.0, 4.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 5.0);
    }

    #[test]
    fn scalar_pythagorean_5_12_13() {
        let v = make_velocity(5.0, 12.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 13.0);
    }

    #[test]
    fn scalar_negative_components() {
        let v = make_velocity(-3.0, -4.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 5.0);
    }

    #[test]
    fn scalar_mixed_sign_components() {
        let v = make_velocity(-3.0, 4.0);
        assert_relative_eq!(v.scalar().get::<meter_per_second>(), 5.0);
    }

    #[test]
    fn scalar_is_always_non_negative() {
        let v = make_velocity(-100.0, -200.0);
        assert!(v.scalar().get::<meter_per_second>() >= 0.0);
    }

    // --- proptest property-based tests ---

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn scalar_is_non_negative_for_all_inputs(
                x in -1.0e12_f64..1.0e12,
                y in -1.0e12_f64..1.0e12,
            ) {
                let v = make_velocity(x, y);
                let s = v.scalar().get::<meter_per_second>();
                prop_assert!(s >= 0.0, "scalar was {} for vx={}, vy={}", s, x, y);
                prop_assert!(s.is_finite(), "scalar was infinite for vx={}, vy={}", x, y);
            }
        }
    }

    // --- SimRate ---

    #[test]
    fn sim_rate_default_is_one() {
        let rate = SimRate::default();
        assert_relative_eq!(rate.0, 1.0);
    }

    #[test]
    fn sim_rate_constants() {
        assert_relative_eq!(SimRate::MIN, 0.25);
        assert_relative_eq!(SimRate::MAX, 2.0);
        assert_relative_eq!(SimRate::STEP, 0.25);
    }

    // --- GridVisible ---

    #[test]
    fn grid_visible_default_is_true() {
        let gv = GridVisible::default();
        assert!(gv.0);
    }
}
