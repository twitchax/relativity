use super::{
    constants::{PLANET_SPRITE_WIDTH_PX, ROCKET_SPRITE_WIDTH_PX, SCREEN_HEIGHT_UOM, SCREEN_WIDTH_UOM},
    types::{Position, Radius},
};
use crate::shared::{SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX};
use bevy::prelude::*;
use uom::si::f64::Length as UomLength;

#[must_use]
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

#[must_use]
pub fn get_translation_from_position(position: &Position) -> Vec3 {
    let x = (position.x / *SCREEN_WIDTH_UOM).value;
    let y = (position.y / *SCREEN_HEIGHT_UOM).value;

    get_translation_from_percentage(x, y)
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_translation_from_percentage(x: f64, y: f64) -> Vec3 {
    let x = (SCREEN_WIDTH_PX * x) as f32;
    let y = (SCREEN_HEIGHT_PX * y) as f32;

    Vec3::new(x, y, 0.0)
}

#[must_use]
pub fn get_position_from_percentage(x: f64, y: f64) -> Position {
    let x = *SCREEN_WIDTH_UOM * x;
    let y = *SCREEN_HEIGHT_UOM * y;

    Position { x, y }
}

#[must_use]
pub fn length_to_pixel(length: UomLength) -> f64 {
    let galaxy_length = *SCREEN_WIDTH_UOM;
    let length_percent = length / galaxy_length;

    length_percent.value * SCREEN_WIDTH_PX
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn planet_sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / PLANET_SPRITE_WIDTH_PX) as f32)
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn rocket_sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / ROCKET_SPRITE_WIDTH_PX) as f32)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::length::kilometer;

    fn make_position(x_km: f64, y_km: f64) -> Position {
        Position {
            x: UomLength::new::<kilometer>(x_km),
            y: UomLength::new::<kilometer>(y_km),
        }
    }

    fn make_radius(r_km: f64) -> Radius {
        Radius { value: UomLength::new::<kilometer>(r_km) }
    }

    // --- has_collided ---

    #[test]
    fn has_collided_overlapping_returns_true() {
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(10.0);
        let pos_b = make_position(5.0, 0.0);
        let rad_b = make_radius(10.0);

        assert!(has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    #[test]
    fn has_collided_distant_returns_false() {
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(1.0);
        let pos_b = make_position(100.0, 0.0);
        let rad_b = make_radius(1.0);

        assert!(!has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    #[test]
    fn has_collided_exact_boundary_returns_true() {
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(5.0);
        let pos_b = make_position(10.0, 0.0);
        let rad_b = make_radius(5.0);

        // distance == sum of radii, should be considered colliding (<=)
        assert!(has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    #[test]
    fn has_collided_same_position_returns_true() {
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(1.0);
        let pos_b = make_position(0.0, 0.0);
        let rad_b = make_radius(1.0);

        assert!(has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    #[test]
    fn has_collided_is_symmetric() {
        let pos_a = make_position(3.0, 4.0);
        let rad_a = make_radius(2.0);
        let pos_b = make_position(10.0, 10.0);
        let rad_b = make_radius(3.0);

        assert_eq!(has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)), has_collided((&pos_b, &rad_b), (&pos_a, &rad_a)),);
    }

    #[test]
    fn has_collided_zero_radius() {
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(0.0);
        let pos_b = make_position(0.0, 0.0);
        let rad_b = make_radius(0.0);

        // Same position with zero radii: distance (0) <= 0, so collided
        assert!(has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    #[test]
    fn has_collided_diagonal_distance() {
        // 3-4-5 right triangle: distance = 5 km
        let pos_a = make_position(0.0, 0.0);
        let rad_a = make_radius(2.0);
        let pos_b = make_position(3.0, 4.0);
        let rad_b = make_radius(2.0);

        // distance 5.0 > sum of radii 4.0 => no collision
        assert!(!has_collided((&pos_a, &rad_a), (&pos_b, &rad_b)));
    }

    // --- get_translation_from_percentage ---

    #[test]
    fn translation_from_percentage_origin() {
        let result = get_translation_from_percentage(0.0, 0.0);
        assert_relative_eq!(result.x, 0.0, epsilon = 1e-6);
        assert_relative_eq!(result.y, 0.0, epsilon = 1e-6);
        assert_relative_eq!(result.z, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn translation_from_percentage_full() {
        let result = get_translation_from_percentage(1.0, 1.0);
        #[allow(clippy::cast_possible_truncation)]
        {
            assert_relative_eq!(result.x, SCREEN_WIDTH_PX as f32, epsilon = 1e-3);
            assert_relative_eq!(result.y, SCREEN_HEIGHT_PX as f32, epsilon = 1e-3);
        }
    }

    #[test]
    fn translation_from_percentage_half() {
        let result = get_translation_from_percentage(0.5, 0.5);
        #[allow(clippy::cast_possible_truncation)]
        {
            assert_relative_eq!(result.x, (SCREEN_WIDTH_PX * 0.5) as f32, epsilon = 1e-3);
            assert_relative_eq!(result.y, (SCREEN_HEIGHT_PX * 0.5) as f32, epsilon = 1e-3);
        }
    }

    // --- get_translation_from_position ---

    #[test]
    fn translation_from_position_origin() {
        let pos = make_position(0.0, 0.0);
        let result = get_translation_from_position(&pos);
        assert_relative_eq!(result.x, 0.0, epsilon = 1e-6);
        assert_relative_eq!(result.y, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn translation_from_position_at_screen_edge() {
        // Position at SCREEN_WIDTH_UOM, SCREEN_HEIGHT_UOM should map to full screen pixels
        let pos = get_position_from_percentage(1.0, 1.0);
        let result = get_translation_from_position(&pos);
        #[allow(clippy::cast_possible_truncation)]
        {
            assert_relative_eq!(result.x, SCREEN_WIDTH_PX as f32, epsilon = 1e-1);
            assert_relative_eq!(result.y, SCREEN_HEIGHT_PX as f32, epsilon = 1e-1);
        }
    }

    // --- get_position_from_percentage ---

    #[test]
    fn position_from_percentage_origin() {
        let pos = get_position_from_percentage(0.0, 0.0);
        assert_relative_eq!(pos.x.value, 0.0, epsilon = 1e-10);
        assert_relative_eq!(pos.y.value, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn position_from_percentage_full() {
        let pos = get_position_from_percentage(1.0, 1.0);
        assert_relative_eq!(pos.x.value, SCREEN_WIDTH_UOM.value, epsilon = 1e-6);
        assert_relative_eq!(pos.y.value, SCREEN_HEIGHT_UOM.value, epsilon = 1e-6);
    }

    #[test]
    fn position_from_percentage_half() {
        let pos = get_position_from_percentage(0.5, 0.5);
        assert_relative_eq!(pos.x.value, SCREEN_WIDTH_UOM.value * 0.5, epsilon = 1e-6);
        assert_relative_eq!(pos.y.value, SCREEN_HEIGHT_UOM.value * 0.5, epsilon = 1e-6);
    }

    // --- length_to_pixel ---

    #[test]
    fn length_to_pixel_zero() {
        let result = length_to_pixel(UomLength::new::<kilometer>(0.0));
        assert_relative_eq!(result, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn length_to_pixel_full_width() {
        let result = length_to_pixel(*SCREEN_WIDTH_UOM);
        assert_relative_eq!(result, SCREEN_WIDTH_PX, epsilon = 1e-6);
    }

    #[test]
    fn length_to_pixel_half_width() {
        let half_width = *SCREEN_WIDTH_UOM * 0.5;
        let result = length_to_pixel(half_width);
        assert_relative_eq!(result, SCREEN_WIDTH_PX * 0.5, epsilon = 1e-6);
    }

    // --- planet_sprite_pixel_radius_to_scale ---

    #[test]
    fn planet_sprite_scale_at_half_sprite_width() {
        // pixels = PLANET_SPRITE_WIDTH_PX / 2 => scale = 1.0
        let result = planet_sprite_pixel_radius_to_scale(PLANET_SPRITE_WIDTH_PX / 2.0);
        assert_relative_eq!(result.x, 1.0, epsilon = 1e-6);
        assert_relative_eq!(result.y, 1.0, epsilon = 1e-6);
        assert_relative_eq!(result.z, 1.0, epsilon = 1e-6);
    }

    #[test]
    fn planet_sprite_scale_zero() {
        let result = planet_sprite_pixel_radius_to_scale(0.0);
        assert_relative_eq!(result.x, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn planet_sprite_scale_proportional() {
        let scale_a = planet_sprite_pixel_radius_to_scale(100.0);
        let scale_b = planet_sprite_pixel_radius_to_scale(200.0);
        assert_relative_eq!(f64::from(scale_b.x) / f64::from(scale_a.x), 2.0, epsilon = 1e-4);
    }

    // --- rocket_sprite_pixel_radius_to_scale ---

    #[test]
    fn rocket_sprite_scale_at_half_sprite_width() {
        // pixels = ROCKET_SPRITE_WIDTH_PX / 2 => scale = 1.0
        let result = rocket_sprite_pixel_radius_to_scale(ROCKET_SPRITE_WIDTH_PX / 2.0);
        assert_relative_eq!(result.x, 1.0, epsilon = 1e-6);
        assert_relative_eq!(result.y, 1.0, epsilon = 1e-6);
        assert_relative_eq!(result.z, 1.0, epsilon = 1e-6);
    }

    #[test]
    fn rocket_sprite_scale_zero() {
        let result = rocket_sprite_pixel_radius_to_scale(0.0);
        assert_relative_eq!(result.x, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn rocket_sprite_scale_proportional() {
        let scale_a = rocket_sprite_pixel_radius_to_scale(50.0);
        let scale_b = rocket_sprite_pixel_radius_to_scale(100.0);
        assert_relative_eq!(f64::from(scale_b.x) / f64::from(scale_a.x), 2.0, epsilon = 1e-4);
    }
}
