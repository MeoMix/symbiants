use std::ops::Add;

use super::{
    emote::{Emote, EmoteType},
    Ant, AntColor, AntInventory, AntLabel, AntName, AntOrientation, AntRole, Dead,
};
use crate::{
    settings::Settings,
    story::{
        common::position::Position,
        element::{
            ui::{get_element_index, ElementExposure, ElementTextureAtlasHandle},
            Element,
        },
        grid::Grid,
        nest_simulation::{
            nest::{AtNest, Nest},
            ModelViewEntityMap,
        },
        story_time::DEFAULT_TICKS_PER_SECOND,
    },
};
use bevy::{prelude::*, utils::{HashSet, HashMap}};

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

fn spawn_ant_sprite(
    commands: &mut Commands,
    model_entity: Entity,
    position: &Position,
    color: &AntColor,
    orientation: &AntOrientation,
    role: &AntRole,
    inventory: &AntInventory,
    dead: Option<&Dead>,
    asset_server: &Res<AssetServer>,
    elements_query: &Query<&Element>,
    grid: &Grid,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) -> Entity {
    // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise.
    let translation_offset = TranslationOffset(Vec3::new(0.0, 0.0, 1.0));

    let (sprite_image, sprite_color) = if dead.is_some() {
        ("images/ant_dead.png", Color::GRAY)
    } else {
        ("images/ant.png", color.0)
    };

    let ant_view_entity = commands
        .spawn((
            translation_offset,
            SpriteBundle {
                texture: asset_server.load(sprite_image),
                sprite: Sprite {
                    color: sprite_color,
                    // 1.5 is just a feel good number to make ants slightly larger than the elements they dig up
                    custom_size: Some(Vec2::splat(1.5)),
                    ..default()
                },
                transform: Transform {
                    translation: grid
                        .grid_to_world_position(*position)
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
                &elements_query,
                &element_texture_atlas_handle,
            ) {
                parent.spawn(bundle);
            }

            if *role == AntRole::Queen {
                parent.spawn(SpriteBundle {
                    texture: asset_server.load("images/crown.png"),
                    transform: Transform::from_xyz(0.33, 0.33, 1.0),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(0.5)),
                        ..default()
                    },
                    ..default()
                });
            }
        })
        .id();

    model_view_entity_map
        .0
        .insert(model_entity, ant_view_entity);

    ant_view_entity
}

fn spawn_ant_label_text2d(
    commands: &mut Commands,
    position: &Position,
    name: &AntName,
    ant_view_entity: Entity,
    grid: &Grid,
) {
    // TODO: z-index is 1.0 here because label gets hidden behind dirt/sand otherwise.
    let translation_offset = TranslationOffset(Vec3::new(0.0, -1.0, 1.0));

    commands.spawn((
        translation_offset,
        Text2dBundle {
            transform: Transform {
                translation: grid
                    .grid_to_world_position(*position)
                    .add(translation_offset.0),
                // TODO: This is an unreasonably small value for text, but is needed for crisp rendering. Does that mean I am doing something wrong?
                scale: Vec3::new(0.01, 0.01, 0.0),
                ..default()
            },
            text: Text::from_section(
                name.0.as_str(),
                TextStyle {
                    color: Color::WHITE,
                    font_size: 60.0,
                    ..default()
                },
            ),
            ..default()
        },
        AntLabel(ant_view_entity),
        AtNest,
    ));
}

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
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = nest_query.single();

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in &ants_query
    {
        let ant_view_entity = spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            orientation,
            role,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );

        spawn_ant_label_text2d(&mut commands, position, name, ant_view_entity, &grid);
    }
}

pub fn rerender_ants(
    ant_model_query: Query<
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
        With<AtNest>,
    >,
    label_query: Query<Entity, With<AntLabel>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = nest_query.single();

    // TODO: This wouldn't be necessary if Ant maintained LabelEntity somewhere?
    for label_entity in label_query.iter() {
        commands.entity(label_entity).despawn();
    }

    // TODO: better approach?
    let entries = model_view_entity_map
        .0
        .iter()
        .map(|(&model_entity, &view_entity)| (model_entity, view_entity))
        .collect::<Vec<_>>();
    for (model_entity, view_entity) in entries {
        commands.entity(view_entity).despawn_recursive();
        model_view_entity_map.0.remove(&model_entity);
    }

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in
        ant_model_query.iter()
    {
        let ant_view_entity = spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            orientation,
            role,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_atlas_handle,
            &mut model_view_entity_map,
        );

        spawn_ant_label_text2d(&mut commands, position, name, ant_view_entity, &grid);
    }
}

// TODO: invert ownership on label so that this can be O(1) instead of O(n).
pub fn on_despawn_ant(
    mut removed: RemovedComponents<Ant>,
    label_query: Query<(Entity, &AntLabel)>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let ant_model_entities = &mut removed.read().collect::<HashSet<_>>();

    {
        let ant_view_entities = ant_model_entities
            .iter()
            .filter_map(|model_entity| model_view_entity_map.0.get(model_entity))
            .collect::<HashSet<_>>();

        for (label_entity, ant_label) in label_query.iter() {
            if ant_view_entities.contains(&ant_label.0) {
                commands.entity(label_entity).despawn();
            }
        }
    }

    for ant_model_entity in ant_model_entities.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.0.remove(ant_model_entity) {
            commands.entity(ant_view_entity).despawn();
        }
    }
}

pub fn on_update_ant_inventory(
    mut commands: Commands,
    ant_model_query: Query<(Entity, &AntInventory), Changed<AntInventory>>,
    ant_view_query: Query<Option<&Children>>,
    inventory_item_sprite_query: Query<&InventoryItemSprite>,
    elements_query: Query<&Element>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for (ant_model_entity, inventory) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            if let Some(inventory_item_bundle) = get_inventory_item_sprite_bundle(
                &inventory,
                &elements_query,
                &element_texture_atlas_handle,
            ) {
                commands
                    .entity(ant_view_entity)
                    .with_children(|ant: &mut ChildBuilder| {
                        // TODO: store entity somewhere and despawn using it rather than searching
                        ant.spawn(inventory_item_bundle);
                    });
            } else {
                if let Ok(children) = ant_view_query.get(ant_view_entity) {
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
}

#[derive(Component)]
pub struct InventoryItemSprite;

#[derive(Bundle)]
pub struct AntHeldElementSpriteBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    inventory_item_sprite: InventoryItemSprite,
}

fn get_inventory_item_sprite_bundle(
    inventory: &AntInventory,
    elements_query: &Query<&Element>,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
) -> Option<AntHeldElementSpriteBundle> {
    let element_entity = match inventory.0 {
        Some(element_entity) => element_entity,
        None => return None,
    };

    let element = elements_query.get(element_entity).unwrap();

    let element_exposure = ElementExposure {
        north: true,
        east: true,
        south: true,
        west: true,
    };

    let mut sprite = TextureAtlasSprite::new(get_element_index(element_exposure, *element));
    sprite.custom_size = Some(Vec2::splat(1.0));

    let sprite_sheet_bundle = SpriteSheetBundle {
        transform: Transform::from_xyz(1.0, 0.25, 1.0),
        sprite,
        texture_atlas: element_texture_atlas_handle.0.clone(),
        ..default()
    };

    Some(AntHeldElementSpriteBundle {
        sprite_sheet_bundle,
        inventory_item_sprite: InventoryItemSprite,
    })
}

pub fn on_update_ant_position(
    ant_model_query: Query<(Entity, &Position), (With<Ant>, Changed<Position>)>,
    mut ant_view_query: Query<(&mut Transform, &TranslationOffset), Without<AntLabel>>,
    mut ant_label_view_query: Query<
        (&mut Transform, &TranslationOffset, &AntLabel),
        With<AntLabel>,
    >,
    nest_query: Query<&Grid, With<Nest>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    let grid = nest_query.single();
    // TODO: refactor and get rid of this mapping
    let mut view_entity_position_map = HashMap::new();

    for (ant_model_entity, position) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            if let Ok((mut transform, translation_offset)) = ant_view_query.get_mut(ant_view_entity)
            {
                transform.translation = grid
                    .grid_to_world_position(*position)
                    .add(translation_offset.0);
            }

            view_entity_position_map.insert(ant_view_entity, *position);
        }
    }

    // TODO: This seems bad for performance because it iterates all labels each time rather than just focusing on which ant positions changed.
    // Labels are positioned relative to their linked entity (stored at Label.0) and don't have a position of their own
    for (mut transform, translation_offset, label) in ant_label_view_query.iter_mut() {
        if let Some(position) = view_entity_position_map.get(&label.0) {
            transform.translation = grid
                .grid_to_world_position(*position)
                .add(translation_offset.0);
        }
    }
}

pub fn on_update_ant_color(
    ant_model_query: Query<(Entity, &AntColor), (Changed<AntColor>, Without<Dead>)>,
    mut ant_view_query: Query<&mut Sprite>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for (ant_model_entity, color) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            if let Ok(mut sprite) = ant_view_query.get_mut(*ant_view_entity) {
                sprite.color = color.0;
            }
        }
    }
}

pub fn on_update_ant_orientation(
    ant_model_query: Query<(Entity, &AntOrientation), Changed<AntOrientation>>,
    mut ant_view_query: Query<&mut Transform>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for (ant_model_entity, orientation) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            if let Ok(mut transform) = ant_view_query.get_mut(*ant_view_entity) {
                transform.scale = orientation.as_world_scale();
                transform.rotation = orientation.as_world_rotation();
            }
        }
    }
}

pub fn on_added_ant_dead(
    ant_model_query: Query<Entity, Added<Dead>>,
    mut ant_view_query: Query<(&mut Handle<Image>, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for ant_model_entity in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            if let Ok((mut image_handle, mut sprite)) = ant_view_query.get_mut(*ant_view_entity) {
                *image_handle = asset_server.load("images/ant_dead.png");

                // Apply gray tint to dead ants.
                sprite.color = Color::GRAY;
            }
        }
    }
}

// TODO: Invert ownership would make this O(1) instead of O(n).
pub fn on_removed_emote(
    mut removed: RemovedComponents<Emote>,
    emote_view_query: Query<(Entity, &EmoteSprite)>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    let emoting_model_entities = &mut removed.read().collect::<HashSet<_>>();
    let emoting_view_entities = emoting_model_entities
        .iter()
        .filter_map(|model_entity| model_view_entity_map.0.get(model_entity))
        .collect::<HashSet<_>>();

    for (emote_view_entity, emote_sprite) in emote_view_query.iter() {
        if emoting_view_entities.contains(&emote_sprite.parent_entity) {
            // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
            commands.entity(emote_view_entity).remove_parent().despawn();
        }
    }
}

pub fn on_tick_emote(
    mut ant_model_query: Query<(Entity, &mut Emote), With<AtNest>>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    for (ant_model_entity, mut emote) in ant_model_query.iter_mut() {
        let rate_of_emote_expire =
            emote.max() / (settings.emote_duration * DEFAULT_TICKS_PER_SECOND) as f32;
        emote.tick(rate_of_emote_expire);

        if emote.is_expired() {
            commands.entity(ant_model_entity).remove::<Emote>();
        }
    }
}

#[derive(Component)]
pub struct EmoteSprite {
    parent_entity: Entity,
}

pub fn on_added_ant_emote(
    ant_model_query: Query<(Entity, &Emote), Added<Emote>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for (ant_model_entity, emote) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            commands.entity(ant_view_entity).with_children(|parent| {
                let texture = match emote.emote_type {
                    EmoteType::Asleep => asset_server.load("images/zzz.png"),
                    EmoteType::FoodLove => asset_server.load("images/foodlove.png"),
                };

                parent.spawn((
                    EmoteSprite {
                        parent_entity: ant_view_entity,
                    },
                    SpriteBundle {
                        transform: Transform::from_xyz(0.75, 1.0, 1.0),
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(1.0)),
                            ..default()
                        },
                        texture,
                        ..default()
                    },
                ));
            });
        }
    }
}
