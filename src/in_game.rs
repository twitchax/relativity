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

const MASS_FACTOR: f32 = 10_000_000.0f32;
const DAYS_PER_SECOND: f32 = 1.0f32;

const SCREEN_WIDTH: f32 = 1280.0f32;
const SCREEN_HEIGHT: f32 = 720.0f32;
const SPRITE_DIM: f32 = 1280.0f32;

const UNIT_RADIUS_KM: f32 = 60_000_000.0f32;

const MASS_OF_SUN_KG: f32 = MASS_FACTOR * 1.989e30f32;
const MASS_OF_EARTH_KG: f32 = MASS_FACTOR * 5.972e24f32;

const SCREEN_WIDTH_KM: f32 = 6_000_000_000.0f32;
const SCREEN_HEIGHT_KM: f32 = SCREEN_WIDTH_KM * SCREEN_HEIGHT / SCREEN_WIDTH;
const MAX_PLAYER_VELOCITY_KMS: f32 = 270_000.0f32; // 99% of c.




const GRAVITATIONAL_CONSTANT: f32 = 6.674e-11f32;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::InGame), (spawn_camera, spawn_player, spawn_planets))
            .add_systems(
                Update,
                (
                    player_launch,
                    scale_update,
                    velocity_update.after(player_launch),
                    position_update.after(velocity_update),
                    translation_update.after(position_update),
                ).run_if(in_state(AppState::InGame)),
            );
    }
}

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("sprites/planets/sphere2.png");

    commands.spawn(PlayerBundle {
        position: get_position_from_percentage(0.3, 0.3),
        radius: Radius {
            value: UomLength::new::<kilometer>(UNIT_RADIUS_KM),
        },
        sprite: SpriteBundle { texture, ..Default::default() },
        ..Default::default()
    });
}

pub fn spawn_planets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // SUN
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.4, 0.4),
        radius: Radius {
            value: UomLength::new::<kilometer>(3.0 * UNIT_RADIUS_KM),
        },
        mass: Mass {
            value: UomMass::new::<kilogram>(MASS_OF_SUN_KG),
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet04.png"),
            ..Default::default()
        },
        ..Default::default()
    });

    // SUN2
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.7, 0.7),
        radius: Radius {
            value: UomLength::new::<kilometer>(3.0 * UNIT_RADIUS_KM),
        },
        mass: Mass {
            value: UomMass::new::<kilogram>(MASS_OF_SUN_KG),
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet04.png"),
            ..Default::default()
        },
        ..Default::default()
    });


    // EARTH
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.28, 0.28),
        radius: Radius {
            value: UomLength::new::<kilometer>(2.0 * UNIT_RADIUS_KM),
        },
        mass: Mass {
            value: UomMass::new::<kilogram>(MASS_OF_EARTH_KG),
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet03.png"),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 0.0);

    commands.spawn(Camera2dBundle { transform, ..Default::default() });
}

pub fn player_launch(
    mouse_input: Res<Input<MouseButton>>,
    mut player_velocity_query: Query<(&Transform, &mut Velocity), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let Ok((player_transform, mut player_velocity)) = player_velocity_query.get_single_mut() else {
        return;
    };

    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    if player_velocity.x.value != 0.0 || player_velocity.y.value != 0.0 {
        return;
    }

    let window = window_query.get_single().unwrap();
    let cursor_position = window.cursor_position().unwrap();
    let cursor_transform = Vec2::new(cursor_position.x, SCREEN_HEIGHT - cursor_position.y);

    let launch_vector = Vec2::new(cursor_transform.x - player_transform.translation.x, cursor_transform.y - player_transform.translation.y);
    let launch_direction = launch_vector.normalize();
    let launch_power = f32::min(0.8 * SCREEN_WIDTH, launch_vector.length()) / (0.8 * SCREEN_WIDTH);

    player_velocity.x = UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS * launch_power * launch_direction.x);
    player_velocity.y = UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS * launch_power * launch_direction.y);
}

pub fn scale_update(mut query: Query<(&mut Transform, &Radius)>) {
    for (mut transform, radius) in query.iter_mut() {
        let scale = sprite_pixel_radius_to_scale(length_to_pixel(radius.value));
        transform.scale = scale;
    }
}

pub fn position_update(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in query.iter_mut() {
        let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds();

        position.x += velocity.x * time_elapsed;
        position.y += velocity.y * time_elapsed;
    }
}

pub fn translation_update(mut query: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = get_translation_from_position(position);
    }
}

pub fn velocity_update(mut query: Query<(&mut Velocity, Entity, &Position), With<Player>>, masses: Query<(Entity, &Position, &Mass)>, time: Res<Time>) {
    #[allow(non_snake_case)]
    let G = GRAVITATIONAL_CONSTANT * UomForce::new::<newton>(1.0) * UomLength::new::<meter>(1.0) * UomLength::new::<meter>(1.0) / (UomMass::new::<kilogram>(1.0) * UomMass::new::<kilogram>(1.0));
    let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds();

    for (mut velocity, entity, position) in query.iter_mut() {
        if velocity.x.value == 0.0 || velocity.y.value == 0.0 {
            continue;
        }

        let mut total_gravitational_acceleration_x = UomAcceleration::new::<meter_per_second_squared>(0.0);
        let mut total_gravitational_acceleration_y = UomAcceleration::new::<meter_per_second_squared>(0.0);

        for (other_entity, other_position, other_mass) in masses.iter() {
            if entity == other_entity {
                continue;
            }

            let direction = Vec2::new((other_position.x - position.x).value, (other_position.y - position.y).value);
            let direction = direction.normalize();

            let delta_x = position.x - other_position.x;
            let delta_y = position.y - other_position.y;
            let distance_squared = delta_x * delta_x + delta_y * delta_y;

            let gravitational_acceleration_x = direction.x * (G * other_mass.value) / distance_squared;
            let gravitational_acceleration_y = direction.y * (G * other_mass.value) / distance_squared;

            total_gravitational_acceleration_x += gravitational_acceleration_x;
            total_gravitational_acceleration_y += gravitational_acceleration_y;
        }

        velocity.x += total_gravitational_acceleration_x * time_elapsed;
        velocity.y += total_gravitational_acceleration_y * time_elapsed;
    }
}

pub fn get_translation_from_position(position: &Position) -> Vec3 {
    let x = (position.x / UomLength::new::<kilometer>(SCREEN_WIDTH_KM)).value;
    let y = (position.y / UomLength::new::<kilometer>(SCREEN_HEIGHT_KM)).value;

    get_translation_from_percentage(x, y)
}

pub fn get_translation_from_percentage(x: f32, y: f32) -> Vec3 {
    let x = SCREEN_WIDTH * x;
    let y = SCREEN_HEIGHT * y;

    Vec3::new(x, y, 0.0)
}

pub fn get_position_from_percentage(x: f32, y: f32) -> Position {
    let x = UomLength::new::<kilometer>(SCREEN_WIDTH_KM * x);
    let y = UomLength::new::<kilometer>(SCREEN_HEIGHT_KM * y);

    Position { x, y }
}

pub fn length_to_pixel(length: UomLength) -> f32 {
    let galaxy_length = UomLength::new::<kilometer>(SCREEN_WIDTH_KM);
    let length_percent = length / galaxy_length;

    length_percent.value * SCREEN_WIDTH
}

pub fn sprite_pixel_radius_to_scale(pixels: f32) -> Vec3 {
    Vec3::splat(2.0 * pixels / SPRITE_DIM)
}
