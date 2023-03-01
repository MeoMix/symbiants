use bevy::{prelude::*, sprite::Anchor};

#[derive(Component)]
struct Dirt;

#[derive(Bundle)]
pub struct DirtBundle {
    sprite_bundle: SpriteBundle,
    dirt: Dirt,
}

impl DirtBundle {
    pub fn new(position: Vec3) -> Self {
        DirtBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.514, 0.396, 0.224),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            dirt: Dirt,
        }
    }
}
