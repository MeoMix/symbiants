use bevy::prelude::*;

use crate::world_map::{position::Position, WorldMap};

#[derive(Component)]
pub struct Background;

fn create_air_sprites(
    width: f32,
    height: f32,
    world_map: &Res<WorldMap>,
) -> Vec<(SpriteBundle, Background)> {
    let mut air_sprites = vec![];

    let dark_blue = Color::rgba(0.0, 0.0, 0.2, 1.0); // Dark blue color
    let sky_blue = Color::rgba(0.529, 0.808, 0.922, 1.0); // Sky blue color
    let power = 3.0; // You can experiment with different values

    for i in 0..(width as i32) {
        for j in 0..(height as i32) {
            let position = Position::new(i as isize, j as isize);

            let t_y = 1.0 - (j as f32 / (height - 1.0) as f32);

            // Use a power function to control the gradient
            let t_y = t_y.powf(power);

            // Interpolate between dark blue and sky blue based on t_y
            let r = sky_blue.r() + (dark_blue.r() - sky_blue.r()) * t_y;
            let g = sky_blue.g() + (dark_blue.g() - sky_blue.g()) * t_y;
            let b = sky_blue.b() + (dark_blue.b() - sky_blue.b()) * t_y;
            let a = sky_blue.a() + (dark_blue.a() - sky_blue.a()) * t_y;

            let air_sprite = SpriteBundle {
                transform: Transform::from_translation(position.as_world_position(&world_map)),
                sprite: Sprite {
                    color: Color::rgba(r, g, b, a),
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                ..default()
            };

            air_sprites.push((air_sprite, Background));
        }
    }

    air_sprites
}

fn create_tunnel_sprites(
    width: f32,
    height: f32,
    y_offset: f32,
    world_map: &Res<WorldMap>,
) -> Vec<(SpriteBundle, Background)> {
    let mut tunnel_sprites = vec![];

    let deepsoil_tunnel_brown = Color::rgba(0.24, 0.186, 0.106, 1.0);
    let topsoil_tunnel_brown = Color::rgba(0.373, 0.290, 0.165, 1.0);
    let power = 3.0; // You can experiment with different values

    for i in 0..(width as i32) {
        for j in 0..(height as i32) {
            let position = Position::new(i as isize, j as isize + y_offset as isize);

            let t_y = 1.0 - (j as f32 / (height - 1.0) as f32);

            // Use a power function to control the gradient
            let t_y = t_y.powf(power);

            // Interpolate between dark blue and sky blue based on t_y
            let r = deepsoil_tunnel_brown.r()
                + (topsoil_tunnel_brown.r() - deepsoil_tunnel_brown.r()) * t_y;
            let g = deepsoil_tunnel_brown.g()
                + (topsoil_tunnel_brown.g() - deepsoil_tunnel_brown.g()) * t_y;
            let b = deepsoil_tunnel_brown.b()
                + (topsoil_tunnel_brown.b() - deepsoil_tunnel_brown.b()) * t_y;
            let a = deepsoil_tunnel_brown.a()
                + (topsoil_tunnel_brown.a() - deepsoil_tunnel_brown.a()) * t_y;

            let tunnel_sprite = SpriteBundle {
                transform: Transform::from_translation(position.as_world_position(&world_map)),
                sprite: Sprite {
                    color: Color::rgba(r, g, b, a),
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                ..default()
            };

            tunnel_sprites.push((tunnel_sprite, Background));
        }
    }

    tunnel_sprites
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(mut commands: Commands, world_map: Res<WorldMap>) {
    let air_height = *world_map.surface_level() as f32 + 1.0;

    commands.spawn_batch(create_air_sprites(
        *world_map.width() as f32,
        air_height,
        &world_map,
    ));

    commands.spawn_batch(create_tunnel_sprites(
        *world_map.width() as f32,
        *world_map.height() as f32 - (*world_map.surface_level() as f32 + 1.0),
        air_height,
        &world_map,
    ));
}

pub fn teardown_background(query: Query<Entity, With<Background>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
