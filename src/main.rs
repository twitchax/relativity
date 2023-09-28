use bevy::{prelude::*, window::PrimaryWindow, render::view::window, transform, input::keyboard};
use uom::si::{f32::{Length, Velocity, Time as UomTime}, length::{meter, kilometer}, velocity::{meter_per_second, kilometer_per_second}, time::{second, hour, day}};

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, spawn_camera)
    .add_systems(Startup, spawn_player)
    .add_systems(Update, player_launch)
    .add_systems(Update, player_movement)
    .run();
}

#[derive(Component, Default)]
pub struct Player {}

#[derive(Component, Default)]
pub struct VelocityVector {
    pub x: Velocity,
    pub y: Velocity,
}

pub fn spawn_player(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>, asset_server: Res<AssetServer>) {
    let window = window_query.get_single().unwrap();

    println!("{}", window.width());
    println!("{}", window.height());

    let transform = get_transform_from_percentage(0.1, 0.2, window).with_scale(Vec3::splat(0.02));
    let texture = asset_server.load("sprites/planets/sphere2.png");

    commands
        .spawn(Player::default())
        .insert(VelocityVector::default())
        .insert(SpriteBundle {
            transform,
            texture,
            ..Default::default()
        });
}

pub fn spawn_camera(mut commands: Commands) {
    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn(Camera2dBundle {
        transform,
        ..Default::default()
    });
}

pub fn player_launch(mouse_input: Res<Input<MouseButton>>, mut player_query: Query<&mut VelocityVector, With<Player>>, time: Res<Time>) {
    let Ok(mut player) = player_query.get_single_mut() else {
        return;
    };

    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    if player.x.value != 0.0 || player.y.value != 0.0 {
        return;
    }

    player.x = Velocity::new::<kilometer_per_second>(10_000.0);
}

pub fn player_movement(mut player_query: Query<(&mut Transform, &VelocityVector), With<Player>>, time: Res<Time>) {
    let (mut player_transform, player_velocity) = player_query.get_single_mut().unwrap();

    // A second of real-world time maps to day.
    let time_elapsed = UomTime::new::<day>(time.delta_seconds());

    let x_pixels = length_to_pixel(player_velocity.x * time_elapsed);
    let y_pixels = length_to_pixel(player_velocity.y * time_elapsed);

    player_transform.translation += Vec3::new(x_pixels, y_pixels, 0.0);
}

pub fn get_transform_from_percentage(x: f32, y: f32, window: &Window) -> Transform {
    let x = window.width() / 2.0 * (2.0 * x - 1.0);
    let y = window.height() / 2.0 * (2.0 * y - 1.0);

    Transform::from_xyz(x, y, 0.0)
}

pub fn length_to_pixel(length: Length) -> f32 {
    let galaxy_length = Length::new::<kilometer>(6_000_000_000.0);
    let length_percent = length / galaxy_length;


    length_percent.value * 1280.0
}