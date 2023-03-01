use bevy::{prelude::*, sprite::Anchor};

// TODO: Should this be more like Element { type: Air }?
#[derive(Component)]
struct Air;

#[derive(Bundle)]
pub struct AirBundle {
    sprite_bundle: SpriteBundle,
    air: Air,
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
            air: Air,
        }
    }
}
