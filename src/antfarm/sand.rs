use bevy::{prelude::*, sprite::Anchor};

#[derive(Component)]
struct Sand;

#[derive(Bundle)]
pub struct SandBundle {
    sprite_bundle: SpriteBundle,
    sand: Sand,
}

impl SandBundle {
    pub fn new(position: Vec3, size: Option<Vec2>) -> Self {
        SandBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.761, 0.698, 0.502),
                    anchor: Anchor::TopLeft,
                    custom_size: size,
                    ..default()
                },
                ..default()
            },
            sand: Sand,
        }
    }
}
