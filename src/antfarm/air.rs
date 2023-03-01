use bevy::{prelude::*, sprite::Anchor};

use super::Element;

#[derive(Component)]
pub struct Air;

#[derive(Bundle)]
pub struct AirBundle {
    pub sprite_bundle: SpriteBundle,
    element: Element,
    air: Air,
}

impl AirBundle {
    pub fn new(position: Vec3) -> Self {
        AirBundle {
            sprite_bundle: SpriteBundle {
                // Air is transparent so reveal background
                visibility: Visibility::INVISIBLE,
                transform: Transform {
                    translation: position,
                    ..default()
                },
                sprite: Sprite {
                    // Fully transparent color - could in theory set to something if air was made visible.
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Air,
            air: Air,
        }
    }
}
