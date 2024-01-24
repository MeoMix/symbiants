pub mod sprite_sheet;

use crate::story::{
    common::position::Position,
    element::{Air, Element, ElementExposure},
    grid::Grid,
    rendering::common::{ModelViewEntityMap, VisibleGrid},
    simulation::nest_simulation::nest::{AtNest, Nest},
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use self::sprite_sheet::{get_element_index, ElementTilemap};

/// When an element's position or exposure changes - redraw its sprite.
/// This will run when an element is first spawned because Changed is true on create.
pub fn on_update_element(
    mut element_query: Query<
        (&Position, &Element, &ElementExposure, Entity),
        (
            Or<(Changed<Position>, Changed<ElementExposure>)>,
            With<AtNest>,
            Without<Air>,
        ),
    >,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
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

    for (position, element, element_exposure, element_model_entity) in element_query.iter_mut() {
        update_element_sprite(
            element_model_entity,
            element,
            position,
            element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

/// When user switches to a different scene (Nest->Crater) all Nest views are despawned.
/// Thus, when switching back to Nest, all Elements need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate rerender system is needed.
pub fn rerender_elements(
    mut element_query: Query<
        (&Position, &Element, &ElementExposure, Entity),
        (With<AtNest>, Without<Air>),
    >,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = nest_query.single();

    for (position, element, element_exposure, entity) in element_query.iter_mut() {
        update_element_sprite(
            entity,
            element,
            position,
            element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

/// When an Element model is despawned, its corresponding view should be despawned as well.
/// Note that a model may be despawned when an unrelated scene is visible. In this scenario,
/// there is no view to despawn, so the ModelViewEntityMap lookup will fail. This is fine.
pub fn on_despawn_element(
    mut removed: RemovedComponents<Element>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    for model_entity in removed.read() {
        if let Some(view_entity) = model_view_entity_map.remove(&model_entity) {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}

pub fn cleanup_elements(
    mut commands: Commands,
    element_tilemap_query: Query<Entity, With<ElementTilemap>>,
) {
    let element_tilemap_entity = element_tilemap_query.single();
    commands.entity(element_tilemap_entity).despawn_recursive();
    // TODO: remove ElementTextureAtlasHandle and ElementSpriteSheetHandle if committing to the full cleanup process
}

/// Non-System Helper Functions:

/// Conditionally spawn or insert an Element based on whether it already exists in the world.
/// TODO: Is there significant performance overhead when inserting a full TileBundle in scenarios where
/// just position has been updated?
fn update_element_sprite(
    element_model_entity: Entity,
    element: &Element,
    element_position: &Position,
    element_exposure: &ElementExposure,
    grid: &Grid,
    commands: &mut Commands,
    tilemap_query: &mut Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    let (tilemap_entity, mut tile_storage) = tilemap_query.single_mut();
    let tile_pos = grid.grid_to_tile_pos(*element_position);

    let tile_bundle = (
        AtNest,
        TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(get_element_index(*element_exposure, *element) as u32),
            ..default()
        },
    );

    // We have the model entity, but need to look for a corresponding view entity. If we have one, then just update it, otherwise create it.
    // TODO: Could provide better enforcement against bad state here if separated this function into spawn vs update.
    if let Some(&element_view_entity) = model_view_entity_map.get(&element_model_entity) {
        commands.entity(element_view_entity).insert(tile_bundle);
        tile_storage.set(&tile_pos, element_view_entity);
    } else {
        let element_view_entity = commands.spawn(tile_bundle).id();
        model_view_entity_map.insert(element_model_entity, element_view_entity);
        tile_storage.set(&tile_pos, element_view_entity);
    }
}
