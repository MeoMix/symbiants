use crate::common::{
    element::{
        sprite_sheet::{
            get_element_index, ElementSpriteSheetHandle, ElementTextureAtlasLayoutHandle,
        },
        ElementExposure,
    },
    visible_grid::{grid_to_world_position, VisibleGrid},
    ModelViewEntityMap,
};
use bevy::{color, prelude::*};
use simulation::{
    common::{
        ant::{Ant, AntColor, AntInventory, AntName, Dead},
        element::Element,
        grid::Grid,
        position::Position,
    },
    crater_simulation::{
        ant::CraterOrientation,
        crater::{AtCrater, Crater},
    },
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

/// When an ant model gains AtCrater render an associated ant sprite.
/// This handles the initial rendering of the ant sprite on load as well as when ants transition between zones.
pub fn on_added_ant_at_crater(
    mut commands: Commands,
    ants_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &CraterOrientation,
            &AntName,
            &AntInventory,
            Option<&Dead>,
        ),
        Added<AtCrater>,
    >,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    crater_query: Query<&Grid, With<Crater>>,
    element_texture_handle: Res<ElementSpriteSheetHandle>,
    element_texture_atlas_layout_handle: Res<ElementTextureAtlasLayoutHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match crater_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (ant_model_entity, position, color, orientation, name, inventory, dead) in &ants_query {
        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
            orientation,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_handle,
            &element_texture_atlas_layout_handle,
            &mut model_view_entity_map,
        );
    }
}

/// When user switches to a different scene (Crater->Nest) all Crater views are despawned.
/// Thus, when switching back to Crater, all Ants need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate spawn system is needed.
pub fn spawn_ants(
    ant_model_query: Query<
        (
            Entity,
            &Position,
            &AntColor,
            &CraterOrientation,
            &AntName,
            &AntInventory,
            Option<&Dead>,
        ),
        With<AtCrater>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    elements_query: Query<&Element>,
    crater_query: Query<&Grid, With<Crater>>,
    element_texture_handle: Res<ElementSpriteSheetHandle>,
    element_texture_atlas_layout_handle: Res<ElementTextureAtlasLayoutHandle>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = crater_query.single();

    for (ant_model_entity, position, color, orientation, name, inventory, dead) in
        ant_model_query.iter()
    {
        spawn_ant_sprite(
            &mut commands,
            ant_model_entity,
            position,
            color,
            name,
            orientation,
            inventory,
            dead,
            &asset_server,
            &elements_query,
            &grid,
            &element_texture_handle,
            &element_texture_atlas_layout_handle,
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
    ant_model_query: Query<(Entity, Ref<AntInventory>, &CraterOrientation), With<AtCrater>>,
    mut ant_view_query: Query<&mut AntSpriteContainer>,
    elements_query: Query<&Element>,
    element_texture_handle: Res<ElementSpriteSheetHandle>,
    element_texture_atlas_layout_handle: Res<ElementTextureAtlasLayoutHandle>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<AtCrater>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, inventory, orientation) in ant_model_query.iter() {
        if inventory.is_added() || !inventory.is_changed() {
            continue;
        }

        // If inventory changed then, if there is an inventory sprite, need to despawn it.
        // Then, if there is inventory currently, need to spawn it.
        if let Some(&ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let mut ant_sprite_container = match ant_view_query.get_mut(ant_view_entity) {
                Ok(ant_sprite_container) => ant_sprite_container,
                Err(_) => {
                    // TODO: This should never happen, but does happen sometimes. I think it occurs when ants transition
                    // from Crater to Nest. Change detection is still responding to something in Crater, but sprite is in another zone.
                    // Safe to ignore, but indicative of an architectural concern.
                    continue;
                }
            };

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
                    &element_texture_handle,
                    &element_texture_atlas_layout_handle,
                    &orientation,
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
    ant_model_query: Query<(Entity, Ref<Position>), (With<Ant>, With<AtCrater>)>,
    mut ant_view_query: Query<(&mut Transform, &TranslationOffset)>,
    grid_query: Query<&Grid, With<AtCrater>>,
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

pub fn on_update_ant_orientation(
    ant_model_query: Query<(Entity, Ref<CraterOrientation>, Option<&Dead>), With<AtCrater>>,
    ant_view_query: Query<&AntSpriteContainer>,
    mut ant_sprite_query: Query<&mut Handle<Image>>,
    mut inventory_transform_query: Query<&mut Transform>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid, With<AtCrater>>,
    asset_server: Res<AssetServer>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (ant_model_entity, orientation, dead) in ant_model_query.iter() {
        if !orientation.is_changed() || orientation.is_added() {
            continue;
        }

        if let Some(ant_view_entity) = model_view_entity_map.get(&ant_model_entity) {
            let ant_sprite_container = ant_view_query.get(*ant_view_entity).unwrap();
            let mut handle = ant_sprite_query
                .get_mut(ant_sprite_container.sprite_entity)
                .unwrap();

            *handle =
                Handle::from(asset_server.load(get_sprite_image(dead.is_some(), *orientation)));

            // Ensure inventory item orientation is updated to match sprite orientation
            if let Some(inventory_item_entity) = ant_sprite_container.inventory_item_entity {
                // TODO: Sometimes this throws if I just unwrap because I don't call apply_deferred within my reactive UI updates layer
                if let Ok(mut transform) = inventory_transform_query.get_mut(inventory_item_entity)
                {
                    *transform = get_inventory_transform(orientation.as_ref());
                }
            }
        }
    }
}

pub fn on_added_ant_dead(
    ant_model_query: Query<Entity, (Added<Dead>, With<AtCrater>)>,
    ant_view_query: Query<&AntSpriteContainer>,
    mut sprite_image_query: Query<(&mut Handle<Image>, &mut Sprite)>,
    asset_server: Res<AssetServer>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<AtCrater>>,
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
            sprite.color = Color::Srgba(color::palettes::basic::GRAY);
        }
    }
}

/// Remove resources, etc.
pub fn cleanup_ants() {}

/// Non-System Helper Functions:

fn get_sprite_image(is_dead: bool, orientation: CraterOrientation) -> String {
    let sprite_image = if is_dead {
        // TODO: Fix this for various orientations?
        "images/ant_dead.png"
    } else {
        match orientation {
            CraterOrientation::Up => "images/ant_face_up.png",
            CraterOrientation::Right => "images/ant_face_right.png",
            CraterOrientation::Down => "images/ant_face_down.png",
            CraterOrientation::Left => "images/ant_face_left.png",
        }
    };

    sprite_image.to_string()
}

fn spawn_ant_sprite(
    commands: &mut Commands,
    model_entity: Entity,
    position: &Position,
    color: &AntColor,
    name: &AntName,
    orientation: &CraterOrientation,
    inventory: &AntInventory,
    dead: Option<&Dead>,
    asset_server: &Res<AssetServer>,
    elements_query: &Query<&Element>,
    grid: &Grid,
    element_texture_handle: &Res<ElementSpriteSheetHandle>,
    element_texture_atlas_layout_handle: &Res<ElementTextureAtlasLayoutHandle>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    // TODO: z-index is 1.0 here because ant can get hidden behind sand otherwise.
    let translation_offset = TranslationOffset(Vec3::new(0.0, 0.0, 1.0));

    let is_dead = dead.is_some();
    let sprite_color = match is_dead {
        true => Color::Srgba(color::palettes::basic::GRAY),
        false => color.0,
    };
    let sprite_image = get_sprite_image(is_dead, *orientation);

    // Spawn AntSprite with child inventory/hat
    let mut ant_sprite = commands.spawn((SpriteBundle {
        texture: asset_server.load(sprite_image),
        sprite: Sprite {
            color: sprite_color,
            // 1.5 is just a feel good number to make ants slightly larger than the elements they dig up
            custom_size: Some(Vec2::splat(1.5)),
            ..default()
        },
        ..default()
    },));

    let mut inventory_item_entity = None;

    ant_sprite.with_children(|parent: &mut ChildBuilder<'_>| {
        if let Some(element_entity) = inventory.0 {
            let bundle = get_inventory_item_bundle(
                element_entity,
                &elements_query,
                &element_texture_handle,
                &element_texture_atlas_layout_handle,
                &orientation,
            );

            inventory_item_entity = Some(parent.spawn(bundle).id());
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
                inventory_item_entity: None,
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
            AtCrater,
        ))
        .push_children(&[sprite_entity, label_entity])
        .id();

    model_view_entity_map.insert(model_entity, ant_view_entity);
}

fn get_inventory_item_bundle(
    element_entity: Entity,
    elements_query: &Query<&Element>,
    element_texture_handle: &Res<ElementSpriteSheetHandle>,
    element_texture_atlas_layout_handle: &Res<ElementTextureAtlasLayoutHandle>,
    orientation: &CraterOrientation,
) -> (SpriteBundle, TextureAtlas) {
    let element = elements_query.get(element_entity).unwrap();

    let element_exposure = ElementExposure {
        north: true,
        east: true,
        south: true,
        west: true,
    };

    let mut sprite = Sprite::default();
    sprite.custom_size = Some(Vec2::splat(1.0));

    (
        SpriteBundle {
            transform: get_inventory_transform(orientation),
            sprite,
            texture: element_texture_handle.0.clone(),
            ..default()
        },
        TextureAtlas {
            layout: element_texture_atlas_layout_handle.0.clone(),
            index: get_element_index(element_exposure, *element),
        },
    )
}

fn get_inventory_transform(orientation: &CraterOrientation) -> Transform {
    let x_offset = match orientation {
        CraterOrientation::Up => 0.0,
        CraterOrientation::Right => 1.0,
        CraterOrientation::Down => 0.0,
        CraterOrientation::Left => -1.0,
    };

    let y_offset = match orientation {
        CraterOrientation::Up => 1.0,
        CraterOrientation::Right => 0.25,
        CraterOrientation::Down => -1.0,
        CraterOrientation::Left => 0.25,
    };

    Transform::from_xyz(x_offset, y_offset, 1.0)
}
