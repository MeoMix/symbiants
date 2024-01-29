pub mod emote;

use crate::{
    common::{ModelViewEntityMap, VisibleGrid},
    nest_rendering::element::sprite_sheet::{get_element_index, ElementTextureAtlasHandle},
};
use bevy::prelude::*;
use simulation::{
    common::{grid::Grid, position::Position},
    nest_simulation::{
        ant::{Ant, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead},
        element::{Element, ElementExposure},
        nest::{AtNest, Nest},
    },
};
use std::ops::Add;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

#[derive(Component)]
pub struct AntSprite;

#[derive(Component)]
pub struct InventoryItemSprite;

#[derive(Bundle)]
pub struct AntHeldElementSpriteBundle {
    sprite_sheet_bundle: SpriteSheetBundle,
    inventory_item_sprite: InventoryItemSprite,
}

/// When an ant model is added to the simulation, render an associated ant sprite.
/// This *only* handles the initial rendering of the ant sprite. Updates are handled by other systems.
/// This does handle rendering the ant's held inventory item, it's role-associated hat, it's name label,
/// and properly draws it as dead if the model is dead when spawned.
/// This does not handle rendering the ant's emote.
/// All of this is a bit dubious because ants aren't expected to spawn dead, or to spawn holding anything, but
/// allows for code reuse when rerendering exists ants after toggling between scenes.
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
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in &ants_query
    {
        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
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
    }
}

/// When user switches to a different scene (Nest->Crater) all Nest views are despawned.
/// Thus, when switching back to Nest, all Ants need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate rerender system is needed.
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = nest_query.single();

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in
        ant_model_query.iter()
    {
        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
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
    }
}

/// When an Ant model picks up or sets down an inventory item (i.e. an Element), its view
/// needs to be updated to reflect the change.
///
/// Note that the root view of an ant is a SpatialBundle container with AntSprite as a child of the SpatialBundle.
/// This is because Label is associated with the Ant's position, but is not associated with the Ant's rotation.
/// In contrast, inventory is held in the Ant's mouth, and is thus affected by the Ant's rotation, so
/// inventory needs to be spawned as a child of AntSprite not as a child of the SpatialBundle container.
pub fn on_update_ant_inventory(
    mut commands: Commands,
    ant_model_query: Query<(Entity, &AntInventory), Changed<AntInventory>>,
    ant_view_query: Query<Option<&Children>>,
    ant_sprite_view_query: Query<&mut Sprite, With<AntSprite>>,
    inventory_item_sprite_query: Query<&InventoryItemSprite>,
    elements_query: Query<&Element>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, inventory) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Some(inventory_item_bundle) = get_inventory_item_sprite_bundle(
                &inventory,
                &elements_query,
                &element_texture_atlas_handle,
            ) {
                if let Ok(children) = ant_view_query.get(ant_view_entity) {
                    if let Some(children) = children {
                        let ant_sprite_entity = children
                            .iter()
                            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                            .unwrap();

                        commands.entity(*ant_sprite_entity).with_children(
                            |ant_sprite: &mut ChildBuilder| {
                                // TODO: store entity somewhere and despawn using it rather than searching
                                ant_sprite.spawn(inventory_item_bundle);
                            },
                        );
                    }
                }
            } else {
                if let Ok(children) = ant_view_query.get(ant_view_entity) {
                    if let Some(children) = children {
                        let ant_sprite_entity = children
                            .iter()
                            .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                            .unwrap();

                        if let Ok(children) = ant_view_query.get(*ant_sprite_entity) {
                            if let Some(children) = children {
                                for &child in children.iter().filter(|&&child| {
                                    inventory_item_sprite_query.get(child).is_ok()
                                }) {
                                    // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
                                    commands.entity(child).remove_parent().despawn();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// When an Ant model moves, its corresponding view needs to be updated.
/// This affects the translation of the SpatialBundle wrapping the AntSprite.
/// This allows for the ant's associated Label to move in sync with the AntSprite.
pub fn on_update_ant_position(
    ant_model_query: Query<(Entity, &Position), (With<Ant>, Changed<Position>)>,
    mut ant_view_query: Query<(&mut Transform, &TranslationOffset)>,
    nest_query: Query<&Grid, With<Nest>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position) in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok((mut transform, translation_offset)) = ant_view_query.get_mut(ant_view_entity)
            {
                transform.translation = grid
                    .grid_to_world_position(*position)
                    .add(translation_offset.0);
            }
        }
    }
}

pub fn on_update_ant_color(
    ant_model_query: Query<(Entity, &AntColor), (Changed<AntColor>, Without<Dead>)>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<&mut Sprite, With<AntSprite>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Grid, With<Nest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, color) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok(mut sprite) = ant_sprite_view_query.get_mut(*ant_sprite_entity) {
                        sprite.color = color.0;
                    }
                }
            }
        }
    }
}

pub fn on_update_ant_orientation(
    ant_model_query: Query<(Entity, &AntOrientation), Changed<AntOrientation>>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<&mut Transform, With<AntSprite>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Grid, With<Nest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, orientation) in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok(mut transform) = ant_sprite_view_query.get_mut(*ant_sprite_entity) {
                        transform.scale = orientation.as_world_scale();
                        transform.rotation = orientation.as_world_rotation();
                    }
                }
            }
        }
    }
}

pub fn on_added_ant_dead(
    ant_model_query: Query<Entity, Added<Dead>>,
    ant_view_query: Query<&Children>,
    mut ant_sprite_view_query: Query<(&mut Handle<Image>, &mut Sprite), With<AntSprite>>,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for ant_model_entity in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok(children) = ant_view_query.get(*ant_view_entity) {
                if let Some(ant_sprite_entity) = children
                    .iter()
                    .find(|&&child| ant_sprite_view_query.get(child).is_ok())
                {
                    if let Ok((mut image_handle, mut sprite)) =
                        ant_sprite_view_query.get_mut(*ant_sprite_entity)
                    {
                        *image_handle = asset_server.load("images/ant_dead.png");

                        // Apply gray tint to dead ants.
                        sprite.color = Color::GRAY;
                    }
                }
            }
        }
    }
}

pub fn cleanup_ants() {
    // TODO: Cleanup anything else related to ants here.
}

/// Non-System Helper Functions:

fn spawn_ant_sprite(
    commands: &mut Commands,
    model_entity: Entity,
    position: &Position,
    color: &AntColor,
    name: &AntName,
    orientation: &AntOrientation,
    role: &AntRole,
    inventory: &AntInventory,
    dead: Option<&Dead>,
    asset_server: &Res<AssetServer>,
    elements_query: &Query<&Element>,
    grid: &Grid,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
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
            SpatialBundle {
                transform: Transform {
                    translation: grid
                        .grid_to_world_position(*position)
                        .add(translation_offset.0),
                    ..default()
                },
                ..default()
            },
            AtNest,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    AntSprite,
                    SpriteBundle {
                        texture: asset_server.load(sprite_image),
                        sprite: Sprite {
                            color: sprite_color,
                            // 1.5 is just a feel good number to make ants slightly larger than the elements they dig up
                            custom_size: Some(Vec2::splat(1.5)),
                            ..default()
                        },
                        transform: Transform {
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
                });

            parent.spawn(Text2dBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, -1.0, 1.0),
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
            });
        })
        .id();

    model_view_entity_map.insert(model_entity, ant_view_entity);
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
