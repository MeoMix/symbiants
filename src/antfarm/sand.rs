use bevy::{prelude::*, sprite::Anchor};

#[derive(Component)]
pub struct Sand;

#[derive(Component)]
pub struct Active(pub bool);

#[derive(Bundle)]
pub struct SandBundle {
    sprite_bundle: SpriteBundle,
    sand: Sand,
    is_active: Active,
}

impl SandBundle {
    pub fn new(position: Vec3, size: Option<Vec2>, is_active: bool) -> Self {
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
            is_active: Active(is_active),
            sand: Sand,
        }
    }
}
