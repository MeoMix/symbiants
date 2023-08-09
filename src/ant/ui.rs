use super::{Ant, AntColor, AntInventory, AntName, AntOrientation, AntRole, InventoryItemBundle, Dead, InventoryItem};
use crate::{
    common::{Label, TranslationOffset},
    element::{Element, ui::get_element_color},
    map::Position,
    time::IsFastForwarding,
};
use bevy::{prelude::*, sprite::Anchor};
use std::ops::Add;

// 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
const ANT_SCALE: f32 = 1.2;

// TODO: despawning ants?
// Handle rendering / display details for ants spawned in the simulation logic.
// This involves showing the ant sprite, anything the ant might be carrying, and its name.
pub fn on_spawn_ant(
    mut commands: Commands,
    ants: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &AntOrientation,
            &AntName,
            &AntInventory,
            &AntRole,
        ),
        Added<Ant>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (entity, position, color, orientation, name, inventory, role) in &ants {
        // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise. This isn't a good way of achieving this.
        // y-offset is to align ant with the ground, but then ant looks weird when rotated if x isn't adjusted.
        let translation_offset = TranslationOffset(Vec3::new(0.5, -0.5, 1.0));

        // TODO: instead of using insert, consider spawning this as a child? unsure if there's benefit there but follows Cart's example here:
        // https://github.com/cart/card_combinator/blob/main/src/game/tile.rs
        commands
            .entity(entity)
            .insert((
                translation_offset,
                SpriteBundle {
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
                },
            ))
            .with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
                if let Some(bundle) = get_inventory_item_bundle(inventory) {
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

        commands.spawn((
            translation_offset,
            Text2dBundle {
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
            Label(entity),
        ));
    }
}

pub fn on_update_ant_inventory(
    mut commands: Commands,
    mut query: Query<(Entity, Ref<AntInventory>, Option<&Children>)>,
    inventory_items_query: Query<&InventoryItem>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (entity, inventory, children) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || inventory.is_changed() {
            if let Some(inventory_item_bundle) = get_inventory_item_bundle(&inventory) {
                commands
                    .entity(entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        ant.spawn(inventory_item_bundle);
                    });
            } else {
                if let Some(children) = children {
                    for &child in children.iter() {
                        if inventory_items_query.get_component::<InventoryItem>(child).is_ok() {
                            commands.entity(child).despawn();
                        }
                    }
                }
            }
        }
    }
}

fn get_inventory_item_bundle(inventory: &AntInventory) -> Option<InventoryItemBundle> {
    let element = match &inventory.0 {
        Some(element) => *element,
        None => return None,
    };

    let sprite_bundle = SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0.5, 0.75, 1.0)),
        sprite: Sprite {
            color: get_element_color(element),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    };

    Some(InventoryItemBundle {
        sprite_bundle,
        element,
        inventory_item: InventoryItem,
    })
}

pub fn on_update_ant_orientation(
    mut query: Query<(&mut Transform, Ref<AntOrientation>)>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if is_fast_forwarding.0 {
        return;
    }

    for (mut transform, orientation) in query.iter_mut() {
        if is_fast_forwarding.is_changed() || orientation.is_changed() {
            transform.scale = orientation.as_world_scale();
            transform.rotation = orientation.as_world_rotation();
        }
    }
}

pub fn on_update_ant_dead(
    mut query: Query<&mut Handle<Image>, Added<Dead>>,
    asset_server: Res<AssetServer>,
) {
    for mut image_handle in query.iter_mut() {
        *image_handle = asset_server.load("images/ant_dead.png");
    }
}
