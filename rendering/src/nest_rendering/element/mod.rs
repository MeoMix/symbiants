pub mod sprite_sheet;

use self::sprite_sheet::{get_element_index, ElementTilemap};
use crate::common::{grid_to_tile_pos, ModelViewEntityMap, VisibleGrid};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use simulation::{
    common::{grid::Grid, position::Position},
    nest_simulation::{
        element::{Air, Element, ElementExposure},
        nest::{AtNest, Nest},
    },
};

/// When an Element model is added to the simulation, render an associated Element sprite.
/// This *only* handles the initial rendering of the Element sprite. Updates are handled by other systems.
pub fn on_spawn_element(
    mut element_query: Query<
        (&Position, &Element, &ElementExposure, Entity),
        (Added<Element>, With<AtNest>, Without<Air>),
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

    // Early exit when Nest isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (element_position, element, element_exposure, element_model_entity) in
        element_query.iter_mut()
    {
        spawn_element_sprite(
            element_model_entity,
            element,
            element_position,
            element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

/// When an Element model has its Position updated, reflect the change in Position by updating the Translation
/// on its associated view. Update TileStorage to reflect the change in position, too.
/// This does not include the initial spawn of the Element model, which is handled by `on_spawn_element`.
/// This relies on Ref<Position> instead of Changed<Position> to be able to filter against `is_added()`
pub fn on_update_element_position(
    element_query: Query<(Ref<Position>, Entity), (With<AtNest>, Without<Air>)>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<&mut TileStorage, With<ElementTilemap>>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when Nest isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    let mut tile_storage = tilemap_query.single_mut();

    for (element_position, element_model_entity) in element_query.iter() {
        // `on_spawn_element` handles `Added<Position>`
        if element_position.is_added() || !element_position.is_changed() {
            continue;
        }

        let element_view_entity = match model_view_entity_map.get(&element_model_entity) {
            Some(&element_view_entity) => element_view_entity,
            None => panic!("Expected to find view entity for model entity."),
        };

        let tile_pos = grid_to_tile_pos(grid, *element_position);
        commands.entity(element_view_entity).insert(tile_pos);
        // NOTE: This leaves the previous `tile_pos` stale, but that's fine because it's just Air which isn't rendered.
        // TODO: Consider benefits of tracking PreviousPosition in Element and using that to clear stale tile_pos.
        tile_storage.set(&tile_pos, element_view_entity);
    }
}

/// When an Element model has its ElementExposure updated, reflect the change in Position by updating the TileTextureIndex
/// on its associated view.
/// This does not include the initial spawn of the Element model, which is handled by `on_spawn_element`.
/// This relies on Ref<ElementExposure> instead of Changed<ElementExposure> to be able to filter against `is_added()`
pub fn on_update_element_exposure(
    element_query: Query<(Ref<ElementExposure>, &Element, Entity), (With<AtNest>, Without<Air>)>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when Nest isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (element_exposure, element, element_model_entity) in element_query.iter() {
        // `on_spawn_element` handles `Added<ElementExposure>`
        if element_exposure.is_added() || !element_exposure.is_changed() {
            continue;
        }

        let element_view_entity = match model_view_entity_map.get(&element_model_entity) {
            Some(&element_view_entity) => element_view_entity,
            None => panic!("Expected to find view entity for model entity."),
        };

        let texture_index = TileTextureIndex(get_element_index(*element_exposure, *element) as u32);

        commands.entity(element_view_entity).insert(texture_index);
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

    for (element_position, element, element_exposure, entity) in element_query.iter_mut() {
        spawn_element_sprite(
            entity,
            element,
            element_position,
            element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

pub fn cleanup_elements() {
    // TODO: remove ElementTextureAtlasHandle and ElementSpriteSheetHandle if committing to the full cleanup process
}

/// Non-System Helper Functions:

/// Spawn an Element Sprite at the given Position. Update ModelViewEntityMap and TileStorage to reflect the new view.
fn spawn_element_sprite(
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
    let tile_pos = grid_to_tile_pos(grid, *element_position);

    let tile_bundle = (
        AtNest,
        TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(get_element_index(*element_exposure, *element) as u32),
            ..default()
        },
    );

    let element_view_entity = commands.spawn(tile_bundle).id();
    model_view_entity_map.insert(element_model_entity, element_view_entity);
    tile_storage.set(&tile_pos, element_view_entity);
}
