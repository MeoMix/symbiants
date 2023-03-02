use bevy::{prelude::*, sprite::Anchor};

use super::Element;

#[derive(Bundle)]
pub struct DirtBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
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
            element: Element::Dirt,
        }
    }
}
