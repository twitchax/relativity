use std::char::MAX;

use bevy::{input::keyboard, math, prelude::*, render::view::window, transform, window::PrimaryWindow};
use uom::si::{
    acceleration::meter_per_second_squared,
    f32::{Acceleration as UomAcceleration, Force as UomForce, Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity},
    force::newton,
    heat_capacity::gram_square_meter_per_second_squared_kelvin,
    length::{kilometer, meter},
    mass::kilogram,
    time::{day, hour, second},
    velocity::{kilometer_per_second, meter_per_second},
};

use crate::{types::{PlayerBundle, Radius, PlanetBundle, Mass, Velocity, Player, Position}, state::AppState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, start.run_if(in_state(AppState::Menu)));
    }
}

pub fn start(
    mut mouse_input: ResMut<Input<MouseButton>>,
    mut state: ResMut<NextState<AppState>>
) {
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    mouse_input.release_all();
    mouse_input.reset_all();

    println!("here");

    state.set(AppState::InGame);
}