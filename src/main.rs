pub mod types;
pub mod state;
pub mod in_game;
pub mod menu;

use std::char::MAX;

use bevy::{input::keyboard, math, prelude::*, render::view::window, transform, window::PrimaryWindow};
use in_game::InGamePlugin;
use menu::MenuPlugin;
use state::AppState;
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



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins(MenuPlugin)
        .add_plugins(InGamePlugin)
        .add_state::<AppState>()
        .run();
}

