use bevy::{prelude::*, sprite::Anchor};

use super::{get_surface_level, settings::Settings, WORLD_HEIGHT, WORLD_WIDTH};

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(parent: &mut ChildBuilder, settings: &Res<Settings>) {
    parent.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                get_surface_level(settings.initial_dirt_percent) as f32 + 1.0,
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });

    parent.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(
                0.0,
                -(get_surface_level(settings.initial_dirt_percent) as f32 + 1.0),
                0.0,
            ),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                WORLD_HEIGHT as f32
                    - (get_surface_level(settings.initial_dirt_percent) as f32 + 1.0),
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });
}
