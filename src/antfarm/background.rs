use bevy::{prelude::*, sprite::Anchor};

use super::WorldState;

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(parent: &mut ChildBuilder, world_state: &Res<WorldState>) {
    parent.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(
                world_state.width as f32,
                world_state.surface_level as f32 + 1.0,
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });

    parent.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, -((world_state.surface_level as f32) + 1.0), 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(
                world_state.width as f32,
                world_state.height as f32 - (world_state.surface_level as f32 + 1.0),
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });
}
