use bevy::{prelude::*, sprite::Anchor};

#[derive(Bundle)]
pub struct AirBundle {
    sprite_bundle: SpriteBundle,
}

impl AirBundle {
    pub fn new(position: Vec3) -> Self {
        AirBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    color: Color::hex("87ceeb").unwrap(),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
        }
    }
}
