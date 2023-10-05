use bevy::prelude::*;
use crate::game::shared::{types::{VelocityGamma, GravitationalGamma, Clock, Position, Velocity, Mass}, constants::{DAYS_PER_SECOND_UOM, C, G}};
use super::shared::Player;

// Components / bundles.

#[derive(Bundle, Default)]
pub struct PlayerClockBundle {
    pub player: Player,
    pub velocity_gamma: VelocityGamma,
    pub gravitational_gamma: GravitationalGamma,
    pub clock: Clock,
    pub clock_text: TextBundle,
}

// Startup systems.

pub fn spawn_player_clock(mut commands: Commands, asset_server: Res<AssetServer>) {
    let clock_text = TextBundle::from_section("t_p = 00.00 γ_v = 1.00 γ_g = 1.00 v_p = 00.00", TextStyle {
        font_size: 40.0,
        font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
        ..Default::default()
    }).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        left: Val::Px(10.0),
        ..Default::default()
    });

    commands.spawn(PlayerClockBundle {
        clock_text,
        ..Default::default()
    });
}

// Systems.

pub fn player_clock_update(mut query: Query<(&mut Clock, &mut VelocityGamma, &mut GravitationalGamma), With<Player>>, player_query: Query<(Entity, &Position, &Velocity), With<Player>>,masses: Query<(Entity, &Position, &Mass)>, time: Res<Time>) {
    let time_elapsed = *DAYS_PER_SECOND_UOM * time.delta_seconds() as f64;

    let (mut clock, mut velocity_gamma, mut gravitational_gamma) = query.single_mut();
    let (player_entity, player_position, player_velocity) = player_query.single();

    // Compute velocity gamma.

    let v_squared_div_c_squared = (player_velocity.x.value * player_velocity.x.value + player_velocity.y.value * player_velocity.y.value) / (*C * *C);
    velocity_gamma.value = 1.0 / (1.0 - v_squared_div_c_squared.value).sqrt();

    // Compute gravitational gamma.

    let mut total_graviational_gamma = 1.0f64;

    for (other_entity, other_position, other_mass) in masses.iter() {
        if player_entity == other_entity {
            continue;
        }

        let delta_x = player_position.x - other_position.x;
        let delta_y = player_position.y - other_position.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let distance = distance_squared.sqrt();

        let mut gravitational_factor = 1.0 - (2.0 * *G * other_mass.value / (*C * *C * distance)).value;

        if gravitational_factor <= 0.01 {
            gravitational_factor = 0.01;
        }

        let gravitational_gamma = 1.0 / gravitational_factor.sqrt();

        total_graviational_gamma *= gravitational_gamma;
    }

    gravitational_gamma.value = total_graviational_gamma;

    clock.value += time_elapsed / velocity_gamma.value / total_graviational_gamma;
}

pub fn player_clock_text_update(mut query: Query<(&mut Text, &Clock, &VelocityGamma, &GravitationalGamma), With<Player>>) {
    let (mut text, clock, velocity_gamma, gravitational_gamma) = query.single_mut();

    let days = clock.value.value / 24.0 / 3600.0;

    text.sections[0].value = format!("t_p = {:2.2} γ_v = {:2.2} γ_g = {:2.2}", days, velocity_gamma.value, gravitational_gamma.value);
}