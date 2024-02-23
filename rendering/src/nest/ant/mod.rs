pub mod emote;

use crate::common::{
    element::{
        sprite_sheet::{get_element_index, ElementTextureAtlasHandle},
        ElementExposure,
    },
    visible_grid::{grid_to_world_position, VisibleGrid},
    ModelViewEntityMap,
};
use bevy::prelude::*;
use simulation::{
    common::{
        ant::{Ant, AntColor, AntInventory, AntName, NestOrientation, AntRole, Dead},
        element::Element,
        grid::Grid,
        position::Position,
    },
    nest_simulation::nest::AtNest,
};
use std::ops::Add;
#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

// TODO: Maybe call this AntView instead?
#[derive(Component)]
pub struct AntSpriteContainer {
    pub sprite_entity: Entity,
    pub label_entity: Entity,
    pub inventory_item_entity: Option<Entity>,
    pub emote_entity: Option<Entity>,
}

/// When an ant model gains AtNest render an associated ant sprite.
/// This handles the initial rendering of the ant sprite on load as well as when ants transition between zones.
pub fn on_added_ant_at_nest(
    mut commands: Commands,
    ants_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &NestOrientation,
            &AntName,
            &AntRole,
            &AntInventory,
            Option<&Dead>,
        ),
        Added<AtNest>,
    >,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    grid_query: Query<&Grid, With<AtNest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    let grid = grid_query.single();

    for (ant_model_entity, position, color, orientation, name, role, inventory, dead) in
        ants_query.iter()
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
/// have not been changed or added, though, so a separate spawn system is needed.
pub fn spawn_ants(
    ant_model_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &NestOrientation,
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
    grid_query: Query<&Grid, With<AtNest>>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = grid_query.single();

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
/// CAREFUL: Simulation can tick multiple times before rendering. So, it's possible for change detection to
/// indicate a change to inventory has occurred, but that *multiple* changes have occurred. Do not implement this
/// as a simple toggle spawn/despawn of the sprite.
///
/// Note that the root view of an ant is a SpatialBundle container with AntSprite as a child of the SpatialBundle.
/// This is because Label is associated with the Ant's position, but is not associated with the Ant's rotation.
/// In contrast, inventory is held in the Ant's mouth, and is thus affected by the Ant's rotation, so
/// inventory needs to be spawned as a child of AntSprite not as a child of the SpatialBundle container.
pub fn on_update_ant_inventory(
    mut commands: Commands,
    ant_model_query: Query<(Entity, Ref<AntInventory>), With<AtNest>>,
    mut ant_view_query: Query<&mut AntSpriteContainer>,
    elements_query: Query<&Element>,
    element_texture_atlas_handle: Res<ElementTextureAtlasHandle>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<AtNest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, inventory) in ant_model_query.iter() {
        if inventory.is_added() || !inventory.is_changed() {
            continue;
        }

        // If inventory changed then, if there is an inventory sprite, need to despawn it.
        // Then, if there is inventory currently, need to spawn it.
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let mut ant_sprite_container = ant_view_query.get_mut(ant_view_entity).unwrap();

            if let Some(inventory_item_entity) = ant_sprite_container.inventory_item_entity {
                // Surprisingly, Bevy doesn't fix parent/child relationship when despawning children, so do it manually.
                commands
                    .entity(inventory_item_entity)
                    .remove_parent()
                    .despawn();
                ant_sprite_container.inventory_item_entity = None;
            }

            if let Some(element_entity) = inventory.0 {
                let inventory_item_bundle = get_inventory_item_bundle(
                    element_entity,
                    &elements_query,
                    &element_texture_atlas_handle,
                );

                let ant_inventory_item_entity = commands.spawn(inventory_item_bundle).id();

                commands
                    .entity(ant_sprite_container.sprite_entity)
                    .push_children(&[ant_inventory_item_entity]);

                ant_sprite_container.inventory_item_entity = Some(ant_inventory_item_entity);
            }
        }
    }
}

/// When an Ant model moves, its corresponding view needs to be updated.
/// This affects the translation of the SpatialBundle wrapping the AntSprite.
/// This allows for the ant's associated Label to move in sync with the AntSprite.
pub fn on_update_ant_position(
    ant_model_query: Query<(Entity, Ref<Position>), (With<Ant>, With<AtNest>)>,
    mut ant_view_query: Query<(&mut Transform, &TranslationOffset)>,
    grid_query: Query<&Grid, With<AtNest>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match grid_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position) in ant_model_query.iter() {
        if position.is_added() || !position.is_changed() {
            continue;
        }

        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            if let Ok((mut transform, translation_offset)) = ant_view_query.get_mut(ant_view_entity)
            {
                transform.translation =
                    grid_to_world_position(grid, *position).add(translation_offset.0);
            }
        }
    }
}

pub fn on_update_ant_color(
    // TODO: Prefer not needing to exclude Dead here?
    ant_model_query: Query<(Entity, Ref<AntColor>), (Without<Dead>, With<AtNest>)>,
    ant_view_query: Query<&AntSpriteContainer>,
    mut sprite_query: Query<&mut Sprite>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid, With<AtNest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, color) in ant_model_query.iter() {
        if !color.is_changed() || color.is_added() {
            continue;
        }

        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let ant_sprite_container = ant_view_query.get(*ant_view_entity).unwrap();
            let mut sprite = sprite_query
                .get_mut(ant_sprite_container.sprite_entity)
                .unwrap();

            sprite.color = color.0;
        }
    }
}

pub fn on_update_ant_orientation(
    ant_model_query: Query<(Entity, Ref<NestOrientation>), With<AtNest>>,
    ant_view_query: Query<&AntSpriteContainer>,
    mut transform_query: Query<&mut Transform>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid, With<AtNest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, orientation) in ant_model_query.iter() {
        if !orientation.is_changed() || orientation.is_added() {
            continue;
        }

        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let ant_sprite_container = ant_view_query.get(*ant_view_entity).unwrap();
            let mut transform = transform_query
                .get_mut(ant_sprite_container.sprite_entity)
                .unwrap();

            transform.scale = orientation.as_world_scale();
            transform.rotation = orientation.as_world_rotation();
        }
    }
}

pub fn on_added_ant_dead(
    ant_model_query: Query<Entity, (Added<Dead>, With<AtNest>)>,
    ant_view_query: Query<&AntSpriteContainer>,
    mut sprite_image_query: Query<(&mut Handle<Image>, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<AtNest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for ant_model_entity in ant_model_query.iter() {
        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let ant_sprite_container = ant_view_query.get(*ant_view_entity).unwrap();
            let (mut image_handle, mut sprite) = sprite_image_query
                .get_mut(ant_sprite_container.sprite_entity)
                .unwrap();

            *image_handle = asset_server.load("images/ant_dead.png");

            // Apply gray tint to dead ants.
            sprite.color = Color::GRAY;
        }
    }
}

/// Remove resources, etc.
pub fn cleanup_ants() {}

/// Non-System Helper Functions:

fn spawn_ant_sprite(
    commands: &mut Commands,
    model_entity: Entity,
    position: &Position,
    color: &AntColor,
    name: &AntName,
    orientation: &NestOrientation,
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

    // Spawn AntSprite with child inventory/hat
    let mut ant_sprite = commands.spawn((SpriteBundle {
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
    },));

    let mut inventory_item_entity = None;

    ant_sprite.with_children(|parent: &mut ChildBuilder<'_, '_, '_>| {
        if let Some(element_entity) = inventory.0 {
            let bundle = get_inventory_item_bundle(
                element_entity,
                &elements_query,
                &element_texture_atlas_handle,
            );

            inventory_item_entity = Some(parent.spawn(bundle).id());
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

    let sprite_entity = ant_sprite.id();

    let ant_label = commands.spawn(Text2dBundle {
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

    let label_entity = ant_label.id();

    let ant_view_entity = commands
        .spawn((
            AntSpriteContainer {
                sprite_entity,
                label_entity,
                inventory_item_entity,
                emote_entity: None,
            },
            translation_offset,
            SpatialBundle {
                transform: Transform {
                    translation: grid_to_world_position(grid, *position).add(translation_offset.0),
                    ..default()
                },
                ..default()
            },
            AtNest,
        ))
        .push_children(&[sprite_entity, label_entity])
        .id();

    model_view_entity_map.insert(model_entity, ant_view_entity);
}

fn get_inventory_item_bundle(
    element_entity: Entity,
    elements_query: &Query<&Element>,
    element_texture_atlas_handle: &Res<ElementTextureAtlasHandle>,
) -> SpriteSheetBundle {
    let element = elements_query.get(element_entity).unwrap();

    let element_exposure = ElementExposure {
        north: true,
        east: true,
        south: true,
        west: true,
    };

    let mut sprite = TextureAtlasSprite::new(get_element_index(element_exposure, *element));
    sprite.custom_size = Some(Vec2::splat(1.0));

    SpriteSheetBundle {
        transform: Transform::from_xyz(1.0, 0.25, 1.0),
        sprite,
        texture_atlas: element_texture_atlas_handle.0.clone(),
        ..default()
    }
}
