use bevy::{prelude::*, sprite::Anchor};

use super::{Active, Element};

#[derive(Component)]
pub struct Sand;

#[derive(Bundle)]
pub struct SandBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    is_active: Active,
    sand: Sand,
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
            element: Element::Sand,
            sand: Sand,
        }
    }
}
