use bevy::prelude::*;

use crate::grid::WorldMap;

#[derive(Component)]
pub struct Background;

fn create_air_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform::from_xyz(0.0, y_offset, 0.0),
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        ..default()
    }
}

fn create_tunnel_sprite(width: f32, height: f32, y_offset: f32) -> SpriteBundle {
    SpriteBundle {
        transform: Transform::from_xyz(0.0, y_offset, 0.0),
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        ..default()
    }
}

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(mut commands: Commands, world_map: Res<WorldMap>) {
    let air_height = *world_map.surface_level() as f32 + 1.0;

    commands.spawn((
        create_air_sprite(
            *world_map.width() as f32,
            air_height,
            (*world_map.height() as f32 / 2.0) - (air_height / 2.0),
        ),
        Background,
    ));

    commands.spawn((
        create_tunnel_sprite(
            *world_map.width() as f32,
            *world_map.height() as f32 - (*world_map.surface_level() as f32 + 1.0),
            -air_height / 2.0,
        ),
        Background,
    ));
}

pub fn cleanup_background(query: Query<Entity, With<Background>>, mut commands: Commands) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
