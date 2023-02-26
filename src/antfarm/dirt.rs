use bevy::{prelude::*, sprite::Anchor};

#[derive(Bundle)]
pub struct DirtBundle {
    sprite_bundle: SpriteBundle,
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
                    color: Color::hex("836539").unwrap(),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
        }
    }
}
