use bevy::{prelude::*, sprite::Anchor};

// TODO: Should this be more like Element { type: Tunnel }?
#[derive(Component)]
struct Tunnel;

#[derive(Bundle)]
pub struct TunnelBundle {
    sprite_bundle: SpriteBundle,
    dirt: Tunnel,
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
                    color: Color::hex("5f4a2a").unwrap(),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            tunnel: Tunnel,
        }
    }
}
