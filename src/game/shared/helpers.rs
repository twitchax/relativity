use super::{
    constants::{PLANET_SPRITE_WIDTH_PX, ROCKET_SPRITE_WIDTH_PX, SCREEN_HEIGHT_UOM, SCREEN_WIDTH_UOM},
    types::{Position, Radius},
};
use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};
use bevy::prelude::*;
use uom::si::f64::Length as UomLength;

pub fn has_collided(a: (&Position, &Radius), b: (&Position, &Radius)) -> bool {
    let a_pos = a.0;
    let b_pos = b.0;
    let a_radius = a.1;
    let b_radius = b.1;

    let dx = a_pos.x - b_pos.x;
    let dy = a_pos.y - b_pos.y;

    let distance = (dx * dx + dy * dy).sqrt();

    distance <= a_radius.value + b_radius.value
}

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
    let y = *SCREEN_HEIGHT_UOM * y;

    Position { x, y }
}

pub fn length_to_pixel(length: UomLength) -> f64 {
    let galaxy_length = *SCREEN_WIDTH_UOM;
    let length_percent = length / galaxy_length;

    length_percent.value * SCREEN_WIDTH_PX
}

pub fn planet_sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / PLANET_SPRITE_WIDTH_PX) as f32)
}

pub fn rocket_sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / ROCKET_SPRITE_WIDTH_PX) as f32)
}
