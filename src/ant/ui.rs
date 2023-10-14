use std::ops::Add;

use super::{Ant, AntColor, AntInventory, AntLabel, AntName, AntOrientation, AntRole, Dead};
use crate::{
    common::{get_entity_from_id, Id},
    element::{ui::get_element_texture, Element},
    story_time::StoryPlaybackState,
    world_map::{position::Position, WorldMap},
};
use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

pub fn on_spawn_ant(
    mut commands: Commands,
    ants_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &AntOrientation,
            &AntName,
            &AntRole,
            &AntInventory,
            Option<&Dead>,
        ),
        Added<Ant>,
    >,
    asset_server: Res<AssetServer>,
    id_query: Query<(Entity, &Id)>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
) {
    for (entity, position, color, orientation, name, role, inventory, dead) in &ants_query {
        // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise. This isn't a good way of achieving this.
        let translation_offset = TranslationOffset(Vec3::new(0.0, 0.0, 1.0));

        let (sprite_image, sprite_color) = if dead.is_some() {
            ("images/ant_dead.png", Color::GRAY)
        } else {
            ("images/ant.png", color.0)
        };

        commands
            .entity(entity)
            .insert((
                translation_offset,
                SpriteBundle {
                    texture: asset_server.load(sprite_image),
                    sprite: Sprite {
                        color: sprite_color,
                        // 1.2 is just a feel good number to make ants slightly larger than the elements they dig up
                        custom_size: Some(Vec2::splat(1.2)),
                        ..default()
                    },
                    transform: Transform {
                        translation: position
                            .as_world_position(&world_map)
                            .add(translation_offset.0),
                        rotation: orientation.as_world_rotation(),
                        scale: orientation.as_world_scale(),
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
                if let Some(bundle) = get_inventory_item_sprite_bundle(
                    inventory,
                    &id_query,
                    &elements_query,
                    &asset_server,
                ) {
                    parent.spawn(bundle);
                }

                if *role == AntRole::Queen {
                    parent.spawn(SpriteBundle {
                        texture: asset_server.load("images/crown.png"),
                        transform: Transform::from_xyz(0.25, 0.5, 1.0),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(0.5)),
                            ..default()
                        },
                        ..default()
                    });
                }
            });

        // TODO: z-index is 1.0 here because label gets hidden behind dirt/sand otherwise. This isn't a good way of achieving this.
        let translation_offset = TranslationOffset(Vec3::new(0.0, -1.0, 1.0));

        commands.spawn((
            translation_offset,
            Text2dBundle {
                transform: Transform {
                    translation: position
                        .as_world_position(&world_map)
                        .add(translation_offset.0),
                    // TODO: This is an unreasonably small value for text, but is needed for crisp rendering. Does that mean I am doing something wrong?
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
            AntLabel(entity),
        ));
    }
}

// TODO: This doesn't seem to work when used inside of a FixedUpdate loop?
// pub fn on_despawn_ant(
//     mut removed: RemovedComponents<Ant>,
//     mut label_query: Query<(Entity, &AntLabel)>,
//     mut commands: Commands,
// ) {
//     for ant_entity in &mut removed {
//         info!("ant entity removed:'{:?}'", ant_entity);

//         // let (label_entity, _) = label_query
//         //     .iter()
//         //     .find(|(_, label)| label.0 == ant_entity)
//         //     .unwrap();

//         // commands.entity(label_entity).despawn();
//     }
// }

pub fn on_update_ant_inventory(
    mut commands: Commands,
    mut query: Query<(Entity, Ref<AntInventory>, Option<&Children>)>,
    inventory_item_sprite_query: Query<&InventoryItemSprite>,
    elements_query: Query<&Element>,
    id_query: Query<(Entity, &Id)>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    asset_server: Res<AssetServer>,
) {
    if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
        return;
    }

    for (entity, inventory, children) in query.iter_mut() {
        if story_playback_state.is_changed() || inventory.is_changed() {
            if let Some(inventory_item_bundle) =
                get_inventory_item_sprite_bundle(&inventory, &id_query, &elements_query, &asset_server)
            {
                commands
                    .entity(entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        // TODO: store entity somewhere and despawn using it rather than searching
                        ant.spawn(inventory_item_bundle);
                    });
            } else {
                if let Some(children) = children {
                    for &child in children
                        .iter()
                        .filter(|&&child| inventory_item_sprite_query.get(child).is_ok())
                    {
                        // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
                        commands.entity(child).remove_parent();
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct InventoryItemSprite;

#[derive(Bundle)]
pub struct AntHeldElementSpriteBundle {
    sprite_bundle: SpriteBundle,
    inventory_item_sprite: InventoryItemSprite,
}

fn get_inventory_item_sprite_bundle(
    inventory: &AntInventory,
    id_query: &Query<(Entity, &Id)>,
    elements_query: &Query<&Element>,
    asset_server: &Res<AssetServer>,
) -> Option<AntHeldElementSpriteBundle> {
    let inventory_item_element_id = match &inventory.0 {
        Some(inventory_item_element_id) => inventory_item_element_id,
        None => return None,
    };

    // TODO: I am surprised this is working
    let inventory_item_element_entity =
        get_entity_from_id(inventory_item_element_id.clone(), &id_query).unwrap();

    let inventory_item_element = elements_query.get(inventory_item_element_entity).unwrap();

    let sprite_bundle = SpriteBundle {
        transform: Transform::from_xyz(1.0, 0.25, 1.0),
        texture: get_element_texture(inventory_item_element, &asset_server),
        sprite: Sprite {
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        },
        ..default()
    };

    Some(AntHeldElementSpriteBundle {
        sprite_bundle,
        inventory_item_sprite: InventoryItemSprite,
    })
}

pub fn on_update_ant_position(
    mut ant_query: Query<
        (Ref<Position>, &mut Transform, &TranslationOffset),
        (With<Ant>, Without<AntLabel>),
    >,
    mut ant_label_query: Query<
        (&mut Transform, &TranslationOffset, &AntLabel),
        (Without<Ant>, With<AntLabel>),
    >,
    story_playback_state: Res<State<StoryPlaybackState>>,
    world_map: Res<WorldMap>,
) {
    if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
        return;
    }

    for (position, mut transform, translation_offset) in ant_query.iter_mut() {
        if story_playback_state.is_changed() || position.is_changed() {
            transform.translation = position
                .as_world_position(&world_map)
                .add(translation_offset.0);
        }
    }

    // Labels are positioned relative to their linked entity (stored at Label.0) and don't have a position of their own
    for (mut transform, translation_offset, label) in ant_label_query.iter_mut() {
        let (position, _, _) = ant_query.get(label.0).unwrap();

        if story_playback_state.is_changed() || position.is_changed() {
            transform.translation = position
                .as_world_position(&world_map)
                .add(translation_offset.0);
        }
    }
}

pub fn on_update_ant_orientation(
    mut query: Query<(&mut Transform, Ref<AntOrientation>)>,
    story_playback_state: Res<State<StoryPlaybackState>>,
) {
    if story_playback_state.get() == &StoryPlaybackState::FastForwarding {
        return;
    }

    for (mut transform, orientation) in query.iter_mut() {
        if story_playback_state.is_changed() || orientation.is_changed() {
            transform.scale = orientation.as_world_scale();
            transform.rotation = orientation.as_world_rotation();
        }
    }
}

pub fn on_update_ant_dead(
    mut query: Query<(&mut Handle<Image>, &mut Sprite), Added<Dead>>,
    asset_server: Res<AssetServer>,
) {
    for (mut image_handle, mut sprite) in query.iter_mut() {
        *image_handle = asset_server.load("images/ant_dead.png");

        // Apply gray tint to dead ants.
        sprite.color = Color::GRAY;
    }
}
