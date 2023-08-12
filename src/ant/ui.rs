use super::{Ant, AntColor, AntName, AntOrientation, AntRole, Dead, InventoryItem};
use crate::{
    common::{Label, TranslationOffset, Id, get_entity_from_id},
    element::{ui::get_element_sprite, Element},
    map::Position,
    time::IsFastForwarding,
};
use bevy::prelude::*;
use std::ops::Add;

// 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
const ANT_SCALE: f32 = 1.2;

pub fn on_spawn_ant(
    mut commands: Commands,
    ants: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &AntOrientation,
            &AntName,
            &AntRole,
        ),
        Added<Ant>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (entity, position, color, orientation, name, role) in &ants {
        info!("on_spawn_ant");
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
                if *role == AntRole::Queen {
                    info!("Spawning crown for queen");
                    parent.spawn(SpriteBundle {
                        texture: asset_server.load("images/crown.png"),
                        transform: Transform::from_translation(Vec3::new(0.25, 0.5, 1.0)),
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

pub fn on_spawn_inventory_item(
    mut commands: Commands,
    mut inventory_items: Query<(Entity, &InventoryItem), Added<InventoryItem>>,
    elements_query: Query<&Element>,
    id_query: Query<(Entity, &Id)>,
) {
    for (entity, inventory_item) in inventory_items.iter_mut() {
        let element = elements_query.get(entity).unwrap();

        // TODO: HACK HACK HACK LOOOL
        let parent_entity = get_entity_from_id(inventory_item.parent_id.clone(), &id_query).unwrap();
        commands.entity(entity).set_parent(parent_entity);

        info!("on_spawn_inventory_item: {:?}", element);

        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0.5, 0.75, 1.0)),
            sprite: get_element_sprite(element),
            ..default()
        });
    }
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
