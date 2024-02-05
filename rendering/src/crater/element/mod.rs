// use self::sprite_sheet::{get_element_index, ElementSpriteSheetHandle};
use crate::{
    common::{
        visible_grid::{grid_to_tile_pos, VisibleGrid},
        ModelViewEntityMap,
    },
    nest::element::sprite_sheet::{get_element_index, ElementSpriteSheetHandle},
};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use simulation::{
    common::{grid::Grid, position::Position},
    crater_simulation::crater::{AtCrater, Crater},
    nest_simulation::element::{Air, Element, ElementExposure},
};

#[derive(Component)]
pub struct ElementTilemap;

pub fn spawn_element_tilemap(
    element_sprite_sheet_handle: Res<ElementSpriteSheetHandle>,
    mut commands: Commands,
) {
    let grid_size = TilemapGridSize { x: 1.0, y: 1.0 };
    let map_type = TilemapType::default();
    let map_size = TilemapSize { x: 144, y: 144 };

    commands.spawn((
        ElementTilemap,
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: TileStorage::empty(map_size),
            texture: TilemapTexture::Single(element_sprite_sheet_handle.0.clone()),
            tile_size: TilemapTileSize { x: 128.0, y: 128.0 },
            physical_tile_size: TilemapPhysicalTileSize { x: 1.0, y: 1.0 },
            // Element tiles go at z: 1 because they should appear above the background which is rendered at z: 0.
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
            ..default()
        },
    ));
}

/// When an Element model is added to the simulation, render an associated Element sprite.
/// This *only* handles the initial rendering of the Element sprite. Updates are handled by other systems.
pub fn on_spawn_element(
    mut element_query: Query<
        (&Position, &Element, &ElementExposure, Entity),
        (Added<Element>, With<AtCrater>, Without<Air>),
    >,
    crater_query: Query<&Grid, With<Crater>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when Crater isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    let grid = match crater_query.get(visible_grid_entity) {
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
            // element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

/// When user switches to a different scene (Crater->Nest) all Crater views are despawned.
/// Thus, when switching back to Crater, all Elements need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate rerender system is needed.
pub fn rerender_elements(
    mut element_query: Query<
        (&Position, &Element, &ElementExposure, Entity),
        (With<AtCrater>, Without<Air>),
    >,
    crater_query: Query<&Grid, With<Crater>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = crater_query.single();

    for (element_position, element, element_exposure, entity) in element_query.iter_mut() {
        spawn_element_sprite(
            entity,
            element,
            element_position,
            // element_exposure,
            &grid,
            &mut commands,
            &mut tilemap_query,
            &mut model_view_entity_map,
        );
    }
}

pub fn cleanup_elements(mut commands: Commands) {
    commands.remove_resource::<ElementSpriteSheetHandle>();
    commands.remove_resource::<ElementSpriteSheetHandle>();
}

/// Non-System Helper Functions:

/// Spawn an Element Sprite at the given Position. Update ModelViewEntityMap and TileStorage to reflect the new view.
fn spawn_element_sprite(
    element_model_entity: Entity,
    element: &Element,
    element_position: &Position,
    // element_exposure: &ElementExposure,
    grid: &Grid,
    commands: &mut Commands,
    tilemap_query: &mut Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    let (tilemap_entity, mut tile_storage) = tilemap_query.single_mut();
    let tile_pos = grid_to_tile_pos(grid, *element_position);

    let element_exposure = ElementExposure {
        north: false,
        east: false,
        south: false,
        west: false,
    };

    let tile_bundle = (
        AtCrater,
        TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(get_element_index(element_exposure, *element) as u32),
            ..default()
        },
    );

    let element_view_entity = commands.spawn(tile_bundle).id();
    model_view_entity_map.insert(element_model_entity, element_view_entity);
    tile_storage.set(&tile_pos, element_view_entity);
}
