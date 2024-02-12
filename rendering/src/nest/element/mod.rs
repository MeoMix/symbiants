pub mod sprite_sheet;

use self::sprite_sheet::{get_element_index, ElementSpriteSheetHandle};
use crate::common::{
    visible_grid::{grid_to_tile_pos, VisibleGrid},
    ModelViewEntityMap,
};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_tilemap::prelude::*;
use simulation::{
    common::{
        element::{Air, Element},
        grid::{Grid, GridElements},
        position::Position,
    },
    nest_simulation::nest::{AtNest, Nest},
};

#[derive(Component)]
pub struct ElementTilemap;

/// For each non-Air element, track whether there is Air to the north, east, south, and west.
/// This is used to determine which sprite to render for each element.
#[derive(Resource, Default)]
pub struct ElementExposureMap(pub HashMap<Entity, ElementExposure>);

// TODO: It feels wrong to use HashMap + Event rather than Component + ChangeDetection
// But if I want to store Component on the View Entity then I'd need to spawn it in a default state and then update it.
// This is possible, but it would show the default state for a frame before the update without a lot of extra work.
#[derive(Event, PartialEq, Copy, Clone, Debug)]
pub struct ElementExposureChangedEvent(pub Entity);

#[derive(Copy, Clone)]
pub struct ElementExposure {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

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
        (&Position, &Element, Entity),
        (Added<Element>, With<AtNest>, Without<Air>),
    >,
    grid_query: Query<&Grid, With<AtNest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    element_exposure_map: Option<Res<ElementExposureMap>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when Nest isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    let grid = match grid_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    let element_exposure_map = match element_exposure_map {
        Some(element_exposure_map) => element_exposure_map,
        None => panic!("Expected ElementExposureMap to exist whenever grid is visible"),
    };

    for (element_position, element, element_model_entity) in element_query.iter_mut() {
        let element_exposure = element_exposure_map.0.get(&element_model_entity).unwrap();

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

/// When user switches to a different scene (Nest->Crater) all Nest views are despawned.
/// Thus, when switching back to Nest, all Elements need to be redrawn once. Their underlying models
/// have not been changed or added, though, so a separate spawn system is needed.
pub fn spawn_elements(
    mut element_query: Query<(&Position, &Element, Entity), (With<AtNest>, Without<Air>)>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    element_exposure_map: Res<ElementExposureMap>,
) {
    let grid = nest_query.single();

    for (element_position, element, element_model_entity) in element_query.iter_mut() {
        let element_exposure = element_exposure_map.0.get(&element_model_entity).unwrap();

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

pub fn process_element_exposure_changed_events(
    mut element_exposure_changed_events: ResMut<Events<ElementExposureChangedEvent>>,
    element_query: Query<&Element, (With<AtNest>, Without<Air>)>,
    element_exposure_map: Option<Res<ElementExposureMap>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid, With<AtNest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when grid isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    let element_exposure_map = match element_exposure_map {
        Some(element_exposure_map) => element_exposure_map,
        None => panic!("Expected ElementExposureMap to exist whenever grid is visible"),
    };

    for event in element_exposure_changed_events.drain() {
        let element_model_entity = event.0;
        let element = element_query.get(element_model_entity).unwrap();
        let element_exposure = element_exposure_map.0.get(&element_model_entity).unwrap();
        let texture_index = TileTextureIndex(get_element_index(*element_exposure, *element) as u32);

        let element_view_entity = match model_view_entity_map.get(&element_model_entity) {
            Some(&element_view_entity) => element_view_entity,
            // It's OK to fail to find here because view might not have spawned yet. View will spawn with correct details.
            None => continue,
        };

        commands.entity(element_view_entity).insert(texture_index);
    }
}

pub fn insert_element_exposure_map(
    elements_query: Query<(Entity, &Position), (Without<Air>, With<AtNest>)>,
    mut commands: Commands,
    grid_elements: GridElements<AtNest>,
) {
    let map: bevy::utils::hashbrown::HashMap<Entity, ElementExposure> = elements_query
        .iter()
        .map(|(entity, &position)| {
            (
                entity,
                ElementExposure {
                    north: grid_elements.is(position - Position::Y, Element::Air),
                    east: grid_elements.is(position + Position::X, Element::Air),
                    south: grid_elements.is(position + Position::Y, Element::Air),
                    west: grid_elements.is(position - Position::X, Element::Air),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    commands.insert_resource(ElementExposureMap(map));
}

pub fn remove_element_exposure_map(mut commands: Commands) {
    commands.remove_resource::<ElementExposureMap>();
}

pub fn initialize_element_resources(mut commands: Commands) {
    commands.init_resource::<Events<ElementExposureChangedEvent>>();
}

pub fn cleanup_elements(mut commands: Commands) {
    // TODO: Should one of these be 'ElementTextureAtlasHandle' and also why is this in both Crater and Nest?
    commands.remove_resource::<ElementSpriteSheetHandle>();
    commands.remove_resource::<ElementSpriteSheetHandle>();
    commands.remove_resource::<ElementExposureMap>();
    commands.remove_resource::<Events<ElementExposureChangedEvent>>();
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

// TODO: Feel like it would be more clear to run this by relying on RemovedComponents + Changed and excluding Air.
/// Eagerly calculate which sides of a given Element are exposed to Air.
/// Run against all elements changing position - this supports recalculating on Element removal by responding to Air being added.
pub fn update_element_exposure_map(
    // TODO: Consider 'Added'?
    changed_elements_query: Query<(Entity, &Position, &Element), Changed<Position>>,
    grid_elements: GridElements<AtNest>,
    element_exposure_map: Option<ResMut<ElementExposureMap>>,
    mut element_exposure_changed_event_writer: EventWriter<ElementExposureChangedEvent>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid, With<AtNest>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    // Early exit when grid isn't visible because there's no view to update.
    // Exit, rather than skipping system run, to prevent change detection from becoming backlogged.
    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    let mut element_exposure_map = match element_exposure_map {
        Some(element_exposure_map) => element_exposure_map,
        None => panic!("Expected ElementExposureMap to exist whenever grid is visible"),
    };

    let mut entities = HashSet::new();

    for (entity, position, element) in changed_elements_query.iter() {
        if *element != Element::Air {
            entities.insert((entity, *position));
        }

        for adjacent_position in position.get_adjacent_positions() {
            if let Some(adjacent_element_entity) = grid_elements.get_entity(adjacent_position) {
                let adjacent_element = grid_elements.element(*adjacent_element_entity);

                if *adjacent_element != Element::Air {
                    entities.insert((*adjacent_element_entity, adjacent_position));
                }
            }
        }
    }

    for (entity, position) in entities {
        element_exposure_map.0.insert(
            entity,
            ElementExposure {
                north: grid_elements.is(position - Position::Y, Element::Air),
                east: grid_elements.is(position + Position::X, Element::Air),
                south: grid_elements.is(position + Position::Y, Element::Air),
                west: grid_elements.is(position - Position::X, Element::Air),
            },
        );

        element_exposure_changed_event_writer.send(ElementExposureChangedEvent(entity));
    }
}
