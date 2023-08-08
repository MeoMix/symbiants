use bevy::{prelude::*, sprite::Anchor};

use crate::map::WorldMap;

fn create_air_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, y_offset, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(width, height)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    }
}

fn create_tunnel_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, y_offset, 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(width, height)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    }
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(mut commands: Commands, world_map: Res<WorldMap>) {
    commands.spawn(create_air_sprite(
        *world_map.width() as f32,
        *world_map.surface_level() as f32 + 1.0,
        0.0,
    ));

    commands.spawn(create_tunnel_sprite(
        *world_map.width() as f32,
        *world_map.height() as f32 - (*world_map.surface_level() as f32 + 1.0),
        -(*world_map.surface_level() as f32 + 1.0),
    ));
}
