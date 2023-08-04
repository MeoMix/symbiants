
use std::ops::Add;
use bevy::prelude::*;
use crate::map::Position;
use super::{AntColor, AntOrientation, AntName, AntInventory, AntRole, Ant, TranslationOffset, Label};

// 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
const ANT_SCALE: f32 = 1.2;

// TODO: despawning ants?
// Handle rendering / display details for ants spawned in the simulation logic.
// This involves showing the ant sprite, anything the ant might be carrying, and its name.
pub fn on_spawn_ant(
    mut commands: Commands,
    ants: Query<(Entity, &Position, &AntColor, &AntOrientation, &AntName, &AntInventory, &AntRole), Added<Ant>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, position, color, orientation, name, inventory, role) in &ants {
        // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise. This isn't a good way of achieving this.
        // y-offset is to align ant with the ground, but then ant looks weird when rotated if x isn't adjusted.
        let translation_offset = TranslationOffset(Vec3::new(0.5, -0.5, 1.0));

        // TODO: instead of using insert, consider spawning this as a child? unsure if there's benefit there but follows Cart's example here:
        // https://github.com/cart/card_combinator/blob/main/src/game/tile.rs
        commands.entity(entity).insert((translation_offset, SpriteBundle {
            texture: asset_server.load("images/ant.png"),
            sprite: Sprite {
                color: color.0,
                custom_size: Some(Vec2::new(ANT_SCALE, ANT_SCALE)),
                ..default()
            },
            transform: Transform {
                translation: position.as_world_position().add(translation_offset.0),
                rotation: orientation.as_world_rotation(),
                scale: orientation.as_world_scale(),
                ..default()
            },
            ..default()
        }))
        .with_children(|parent| {
            if let Some(bundle) = inventory.get_carrying_bundle() {
                parent.spawn(bundle);
            }

            if *role == AntRole::Queen {
                parent.spawn(SpriteBundle {
                    texture: asset_server.load("images/crown.png"),
                    transform: Transform {
                        translation: Vec3::new(0.25, 0.5, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(0.5, 0.5)),
                        ..default()
                    },
                    ..default()
                });
            }
        });

        // TODO: Is this still the right approach?        
        // TODO: z-index is 1.0 here because label gets hidden behind dirt/sand otherwise. This isn't a good way of achieving this.
        let translation_offset = TranslationOffset(Vec3::new(0.5, -1.5, 1.0));

        commands.spawn((translation_offset, Text2dBundle {
            transform: Transform {
                translation: position.as_world_position().add(translation_offset.0),
                scale: Vec3::new(0.01, 0.01, 0.0),
                ..default()
            },
            text: Text::from_section(
                name.0.as_str(),
                TextStyle {
                    color: Color::BLACK,
                    font_size: 60.0,
                    ..default()
                },
            ),
            ..default()
        },
        Label(entity)
    ));
    }

}
