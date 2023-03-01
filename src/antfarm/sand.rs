use bevy::{prelude::*, sprite::Anchor};

// TODO: Should this be more like Element { type: Sand }?
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
                    color: Color::hex("C2B280").unwrap(),
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
