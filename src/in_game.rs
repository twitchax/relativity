use std::char::MAX;

use bevy::{input::keyboard, math, prelude::*, render::view::window, transform, window::PrimaryWindow};
use glam::DVec2;
use uom::si::{
    acceleration::meter_per_second_squared,
    f64::{Acceleration as UomAcceleration, Force as UomForce, Length as UomLength, Mass as UomMass, Time as UomTime, Velocity as UomVelocity},
    force::newton,
    heat_capacity::gram_square_meter_per_second_squared_kelvin,
    length::{kilometer, meter},
    mass::kilogram,
    time::{day, hour, second},
    velocity::{kilometer_per_second, meter_per_second},
};

use crate::{types::{PlayerBundle, Radius, PlanetBundle, Mass, Velocity, Player, Position, ObserverClockBundle, PlayerClockBundle, VelocityGamma, GravitationalGamma, Clock, Observer}, state::{AppState, GameState}};

const MASS_FACTOR: f64 = 100_000_000.0f64;
const DAYS_PER_SECOND: f64 = 0.5f64;

const SCREEN_WIDTH: f64 = 1280.0f64;
const SCREEN_HEIGHT: f64 = 720.0f64;
const SPRITE_DIM: f64 = 1280.0f64;

const UNIT_RADIUS_KM: f64 = 60_000_000.0f64;

const MASS_OF_SUN_KG: f64 = MASS_FACTOR * 1.989e30f64;
const MASS_OF_EARTH_KG: f64 = MASS_FACTOR * 5.972e24f64;

const SCREEN_WIDTH_KM: f64 = 6_000_000_000.0f64;
const SCREEN_HEIGHT_KM: f64 = SCREEN_WIDTH_KM * SCREEN_HEIGHT / SCREEN_WIDTH;
const C_KMS: f64 = 299_792.0f64; // Speed of light in m/s.
const MAX_PLAYER_VELOCITY_KMS: f64 = 0.99 * C_KMS; // 99% of c.




const GRAVITATIONAL_CONSTANT: f64 = 6.674e-11f64;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
            .add_systems(OnEnter(AppState::InGame), (spawn_camera, spawn_observer, spawn_player, spawn_planets))
            .add_systems(Update, scale_update.run_if(in_state(AppState::InGame)))
            .add_systems(Update, (player_launch, translation_update).run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Paused)))
            .add_systems(
                Update,
                (
                    velocity_update,
                    position_update.after(velocity_update),
                    translation_update.after(position_update),
                    observer_clock_update,
                    observer_clock_text_update.after(observer_clock_update),
                    player_clock_update,
                    player_clock_text_update.after(player_clock_update),
                ).run_if(in_state(AppState::InGame)).run_if(in_state(GameState::Running)),
            );
    }
}

pub fn spawn_observer(mut commands: Commands, asset_server: Res<AssetServer>) {
    let clock_text = TextBundle::from_section("t_o = 00.00", TextStyle {
        font_size: 40.0,
        font: asset_server.load("fonts/HackNerdFontMono-Regular.ttf"),
        ..Default::default()
    }).with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        right: Val::Px(10.0),
        ..Default::default()
    });

    commands.spawn(ObserverClockBundle {
        clock_text,
        ..Default::default()
    });
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

pub fn spawn_planets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // SUN
    commands.spawn(PlanetBundle {
        position: get_position_from_percentage(0.5, 0.5),
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
        position: get_position_from_percentage(0.8, 0.7),
        radius: Radius {
            value: UomLength::new::<kilometer>(2.0 * UNIT_RADIUS_KM),
        },
        mass: Mass {
            value: UomMass::new::<kilogram>(0.2 * MASS_OF_SUN_KG),
        },
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/planets/planet05.png"),
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
    let transform = Transform::from_xyz(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0, 0.0);

    commands.spawn(Camera2dBundle { transform, ..Default::default() });
}

pub fn player_launch(
    mouse_input: Res<Input<MouseButton>>,
    mut player_velocity_query: Query<(&Transform, &mut Velocity), With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<NextState<GameState>>
) {
    let Ok((player_transform, mut player_velocity)) = player_velocity_query.get_single_mut() else {
        return;
    };

    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let window = window_query.get_single().unwrap();
    let cursor_position = window.cursor_position().unwrap();
    let cursor_transform = DVec2::new(cursor_position.x as f64, SCREEN_HEIGHT - cursor_position.y as f64);

    let launch_vector = DVec2::new(cursor_transform.x - player_transform.translation.x as f64, cursor_transform.y - player_transform.translation.y as f64);
    let launch_direction = launch_vector.normalize();
    let launch_power = f64::min(0.8 * SCREEN_WIDTH, launch_vector.length()) / (0.8 * SCREEN_WIDTH);

    player_velocity.x = UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS * launch_power * launch_direction.x);
    player_velocity.y = UomVelocity::new::<kilometer_per_second>(MAX_PLAYER_VELOCITY_KMS * launch_power * launch_direction.y);

    state.set(GameState::Running);
}

pub fn scale_update(mut query: Query<(&mut Transform, &Radius)>) {
    for (mut transform, radius) in query.iter_mut() {
        let scale = sprite_pixel_radius_to_scale(length_to_pixel(radius.value));
        transform.scale = scale;
    }
}

pub fn position_update(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut position, velocity) in query.iter_mut() {
        let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds() as f64;

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
    #[allow(non_snake_case)]
    let C = UomVelocity::new::<kilometer_per_second>(C_KMS);
    let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds() as f64;

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

            let direction = DVec2::new((other_position.x - position.x).value, (other_position.y - position.y).value);
            let direction = direction.normalize();

            let delta_x = position.x - other_position.x;
            let delta_y = position.y - other_position.y;
            let distance_squared = delta_x * delta_x + delta_y * delta_y;
            let distance = distance_squared.sqrt();

            let gravitational_acceleration = (G * other_mass.value) / distance_squared;

            let mut relativistic_adjustment = 1.0 - (2.0 * G * other_mass.value / (C * C * distance)).value;

            if relativistic_adjustment <= 0.0 {
                relativistic_adjustment = 0.0;
            }

            let gravitational_acceleration_x = direction.x * gravitational_acceleration * relativistic_adjustment;
            let gravitational_acceleration_y = direction.y * gravitational_acceleration * relativistic_adjustment;

            total_gravitational_acceleration_x += gravitational_acceleration_x;
            total_gravitational_acceleration_y += gravitational_acceleration_y;
        }

        velocity.x += total_gravitational_acceleration_x * time_elapsed;
        velocity.y += total_gravitational_acceleration_y * time_elapsed;

        if velocity.scalar() > UomVelocity::new::<kilometer_per_second>(C_KMS) {
            println!("AHHHHHHHHHHHHHH");
        }
    }
}

pub fn observer_clock_update(mut query: Query<(&mut Clock), With<Observer>>, time: Res<Time>) {
    let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds() as f64;

    let mut clock = query.single_mut();

    clock.value += time_elapsed;
}

pub fn observer_clock_text_update(mut query: Query<(&mut Text, &Clock), With<Observer>>) {
    let (mut text, clock) = query.single_mut();

    let days = clock.value.value / 24.0 / 3600.0;

    text.sections[0].value = format!("t_o = {:2.2}", days);
}

pub fn player_clock_update(mut query: Query<(&mut Clock, &mut VelocityGamma, &mut GravitationalGamma), With<Player>>, player_query: Query<(Entity, &Position, &Velocity), With<Player>>,masses: Query<(Entity, &Position, &Mass)>, time: Res<Time>) {
    #[allow(non_snake_case)]
    let G = GRAVITATIONAL_CONSTANT * UomForce::new::<newton>(1.0) * UomLength::new::<meter>(1.0) * UomLength::new::<meter>(1.0) / (UomMass::new::<kilogram>(1.0) * UomMass::new::<kilogram>(1.0));
    #[allow(non_snake_case)]
    let C = UomVelocity::new::<kilometer_per_second>(C_KMS);
    let time_elapsed = UomTime::new::<day>(DAYS_PER_SECOND) * time.delta_seconds() as f64;

    let (mut clock, mut velocity_gamma, mut gravitational_gamma) = query.single_mut();
    let (player_entity, player_position, player_velocity) = player_query.single();

    // Compute velocity gamma.

    let v_squared_div_c_squared = (player_velocity.x.value * player_velocity.x.value + player_velocity.y.value * player_velocity.y.value) / (C * C);
    let velocity_γ = 1.0 / (1.0 - v_squared_div_c_squared.value).sqrt();
    velocity_gamma.value = velocity_γ;

    // Compute gravitational gamma.

    let mut total_graviational_γ = 1.0f64;

    for (other_entity, other_position, other_mass) in masses.iter() {
        if player_entity == other_entity {
            continue;
        }

        let delta_x = player_position.x - other_position.x;
        let delta_y = player_position.y - other_position.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let distance = distance_squared.sqrt();

        let mut gravitational_factor = 1.0 - (2.0 * G * other_mass.value / (C * C * distance)).value;

        if gravitational_factor <= 0.01 {
            gravitational_factor = 0.01;
        }

        let gravitational_γ = 1.0 / gravitational_factor.sqrt();

        total_graviational_γ *= gravitational_γ;
    }

    gravitational_gamma.value = total_graviational_γ;

    clock.value += time_elapsed / velocity_γ / total_graviational_γ;
}

pub fn player_clock_text_update(mut query: Query<(&mut Text, &Clock, &VelocityGamma, &GravitationalGamma), With<Player>>) {
    let (mut text, clock, velocity_gamma, gravitational_gamma) = query.single_mut();

    let days = clock.value.value / 24.0 / 3600.0;

    text.sections[0].value = format!("t_p = {:2.2} γ_v = {:2.2} γ_g = {:2.2}", days, velocity_gamma.value, gravitational_gamma.value);
}

pub fn get_translation_from_position(position: &Position) -> Vec3 {
    let x = (position.x / UomLength::new::<kilometer>(SCREEN_WIDTH_KM)).value;
    let y = (position.y / UomLength::new::<kilometer>(SCREEN_HEIGHT_KM)).value;

    get_translation_from_percentage(x, y)
}

pub fn get_translation_from_percentage(x: f64, y: f64) -> Vec3 {
    let x = (SCREEN_WIDTH * x) as f32;
    let y = (SCREEN_HEIGHT * y) as f32;

    Vec3::new(x, y, 0.0)
}

pub fn get_position_from_percentage(x: f64, y: f64) -> Position {
    let x = UomLength::new::<kilometer>(SCREEN_WIDTH_KM * x);
    let y = UomLength::new::<kilometer>(SCREEN_HEIGHT_KM * y);

    Position { x, y }
}

pub fn length_to_pixel(length: UomLength) -> f64 {
    let galaxy_length = UomLength::new::<kilometer>(SCREEN_WIDTH_KM);
    let length_percent = length / galaxy_length;

    length_percent.value * SCREEN_WIDTH
}

pub fn sprite_pixel_radius_to_scale(pixels: f64) -> Vec3 {
    Vec3::splat((2.0 * pixels / SPRITE_DIM) as f32)
}
