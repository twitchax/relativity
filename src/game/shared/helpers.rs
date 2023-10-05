use bevy::prelude::*;
use uom::si::f64::Length as UomLength;
use crate::shared::{SCREEN_WIDTH_PX, SCREEN_HEIGHT_PX};
use super::{types::Position, constants::{SCREEN_WIDTH_UOM, SCREEN_HEIGHT_UOM, PLANET_SPRITE_WIDTH_PX}};

pub fn get_translation_from_position(position: &Position) -> Vec3 {
    let x = (position.x / *SCREEN_WIDTH_UOM).value;
    let y = (position.y / *SCREEN_HEIGHT_UOM).value;

    get_translation_from_percentage(x, y)
}

pub fn get_translation_from_percentage(x: f64, y: f64) -> Vec3 {
    let x = (SCREEN_WIDTH_PX * x) as f32;
    let y = (SCREEN_HEIGHT_PX * y) as f32;

    Vec3::new(x, y, 0.0)
}

pub fn get_position_from_percentage(x: f64, y: f64) -> Position {
    let x = *SCREEN_WIDTH_UOM * x;
    let y = *SCREEN_WIDTH_UOM * y;

    Position { x, y }
}

pub fn length_to_pixel(length: UomLength) -> f64 {
    let galaxy_length = *SCREEN_WIDTH_UOM;
    let length_percent = length / galaxy_length;

    length_percent.value * SCREEN_WIDTH_PX
}

pub fn sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / PLANET_SPRITE_WIDTH_PX) as f32)
}
