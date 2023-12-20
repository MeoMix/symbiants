use super::Element;
use crate::{
    app_state::AppState,
    story::{
        common::position::Position,
        grid::Grid,
        nest_simulation::nest::{AtNest, Nest},
    },
};
use bevy::{asset::LoadState, prelude::*};
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct ElementTilemap;

pub fn on_spawn_element(
    added_elements_query: Query<(Entity, &Position, &Element), Added<Element>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
) {
    let grid = nest_query.single();

    for (entity, position, element) in &added_elements_query {
        update_element_sprite(
            entity,
            element,
            position,
            &elements_query,
            &grid,
            &mut commands,
            &mut tilemap_query,
        );

        let adjacent_positions = position.get_adjacent_positions();

        for adjacent_position in adjacent_positions {
            if let Some(adjacent_element_entity) =
                grid.elements().get_element_entity(adjacent_position)
            {
                let adjacent_element = elements_query.get(*adjacent_element_entity).unwrap();

                if *adjacent_element != Element::Air {
                    update_element_sprite(
                        *adjacent_element_entity,
                        adjacent_element,
                        &adjacent_position,
                        &elements_query,
                        &grid,
                        &mut commands,
                        &mut tilemap_query,
                    );
                }
            }
        }
    }
}

fn update_element_sprite(
    element_entity: Entity,
    element: &Element,
    element_position: &Position,
    elements_query: &Query<&Element>,
    grid: &Grid,
    commands: &mut Commands,
    tilemap_query: &mut Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
) {
    if element == &Element::Air {
        return;
    }

    // TODO: maybe make this reactive rather than calculating all the time to avoid insert when no change in exposure is occurring?
    let element_exposure = ElementExposure {
        north: grid.elements().is_element(
            &elements_query,
            *element_position - Position::Y,
            Element::Air,
        ),
        east: grid.elements().is_element(
            &elements_query,
            *element_position + Position::X,
            Element::Air,
        ),
        south: grid.elements().is_element(
            &elements_query,
            *element_position + Position::Y,
            Element::Air,
        ),
        west: grid.elements().is_element(
            &elements_query,
            *element_position - Position::X,
            Element::Air,
        ),
    };

    let (tilemap_entity, mut tile_storage) = tilemap_query.single_mut();
    let tile_pos = grid.grid_to_tile_pos(*element_position);

    // TODO: Consider spawning as separate entity rather than inserting.
    let tile_entity = commands
        .entity(element_entity)
        .insert(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(get_element_index(element_exposure, *element) as u32),
            ..Default::default()
    })
        .id();
    tile_storage.set(&tile_pos, tile_entity);
}

pub fn rerender_elements(
    mut element_query: Query<(&Position, &Element, Entity), With<AtNest>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
) {
    let grid = nest_query.single();

    for (position, element, entity) in element_query.iter_mut() {
        update_element_sprite(
            entity,
            element,
            position,
            &elements_query,
            &grid,
            &mut commands,
            &mut tilemap_query,
        );
    }
}

pub fn on_update_element_position(
    mut element_query: Query<(&Position, &Element, Entity), Changed<Position>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage), With<ElementTilemap>>,
) {
    let grid = nest_query.single();

    for (position, element, entity) in element_query.iter_mut() {
        update_element_sprite(
            entity,
            element,
            position,
            &elements_query,
            &grid,
            &mut commands,
            &mut tilemap_query,
        );

        let adjacent_positions = position.get_adjacent_positions();

        for adjacent_position in adjacent_positions {
            if let Some(adjacent_element_entity) =
                grid.elements().get_element_entity(adjacent_position)
            {
                let adjacent_element = elements_query.get(*adjacent_element_entity).unwrap();

                if *adjacent_element != Element::Air {
                    update_element_sprite(
                        *adjacent_element_entity,
                        adjacent_element,
                        &adjacent_position,
                        &elements_query,
                        &grid,
                        &mut commands,
                        &mut tilemap_query,
                    );
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ElementSpriteSheetHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct ElementTextureAtlasHandle(pub Handle<TextureAtlas>);

pub fn start_load_element_sprite_sheet(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(ElementSpriteSheetHandle(
        asset_server.load::<Image>("textures/element/sprite_sheet.png"),
    ));
}

pub fn check_element_sprite_sheet_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    element_sprite_sheet_handle: Res<ElementSpriteSheetHandle>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let loaded = asset_server.load_state(&element_sprite_sheet_handle.0) == LoadState::Loaded;

    if loaded {
        let texture_atlas = TextureAtlas::from_grid(
            element_sprite_sheet_handle.0.clone(),
            Vec2::splat(128.0),
            3,
            16,
            None,
            None,
        );

        commands.insert_resource(ElementTextureAtlasHandle(
            texture_atlases.add(texture_atlas),
        ));

        let physical_tile_size = TilemapPhysicalTileSize { x: 1.0, y: 1.0 };
        let tile_size = TilemapTileSize { x: 128.0, y: 128.0 };
        let grid_size = TilemapGridSize { x: 1.0, y: 1.0 };
        let map_type = TilemapType::default();
        let map_size = TilemapSize { x: 144, y: 144 };
        let tile_storage = TileStorage::empty(map_size);

        commands.spawn((
            ElementTilemap,
            TilemapBundle {
                grid_size,
                map_type,
                size: map_size,
                storage: tile_storage,
                texture: TilemapTexture::Single(element_sprite_sheet_handle.0.clone()),
                tile_size,
                physical_tile_size,
                // Element tiles go at z: 1 because they should appear above the background which is rendered at z: 0.
                transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0),
                ..default()
            },
        ));

        next_state.set(AppState::TryLoadSave);
    }
}

#[derive(Copy, Clone)]
pub struct ElementExposure {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

// TODO: super hardcoded to the order they appear in sprite_sheet.png
// Spritesheet is organized as:
// 0 - none exposed
// 1 - north exposed
// 2 - east exposed
// 3 - south exposed
// 4 - west exposed
// 5 - north/east exposed
// 6 - east/south exposed
// 7 - south/west exposed
// 8 - west/north exposed
// 9 - north/south exposed
// 10 - east/west exposed
// 11 - north/east/south exposed
// 12 - east/south/west exposed
// 13 - south/west/north exposed
// 14 - west/north/east exposed
// 15 - all exposed
pub fn get_element_index(exposure: ElementExposure, element: Element) -> usize {
    let row_index = match exposure {
        ElementExposure {
            north: false,
            east: false,
            south: false,
            west: false,
        } => 0,
        ElementExposure {
            north: true,
            east: false,
            south: false,
            west: false,
        } => 1,
        ElementExposure {
            north: false,
            east: true,
            south: false,
            west: false,
        } => 2,
        ElementExposure {
            north: false,
            east: false,
            south: true,
            west: false,
        } => 3,
        ElementExposure {
            north: false,
            east: false,
            south: false,
            west: true,
        } => 4,
        ElementExposure {
            north: true,
            east: true,
            south: false,
            west: false,
        } => 5,
        ElementExposure {
            north: false,
            east: true,
            south: true,
            west: false,
        } => 6,
        ElementExposure {
            north: false,
            east: false,
            south: true,
            west: true,
        } => 7,
        ElementExposure {
            north: true,
            east: false,
            south: false,
            west: true,
        } => 8,
        ElementExposure {
            north: true,
            east: false,
            south: true,
            west: false,
        } => 9,
        ElementExposure {
            north: false,
            east: true,
            south: false,
            west: true,
        } => 10,
        ElementExposure {
            north: true,
            east: true,
            south: true,
            west: false,
        } => 11,
        ElementExposure {
            north: false,
            east: true,
            south: true,
            west: true,
        } => 12,
        ElementExposure {
            north: true,
            east: false,
            south: true,
            west: true,
        } => 13,
        ElementExposure {
            north: true,
            east: true,
            south: false,
            west: true,
        } => 14,
        ElementExposure {
            north: true,
            east: true,
            south: true,
            west: true,
        } => 15,
    };

    let column_index = match element {
        Element::Dirt => 0,
        Element::Food => 1,
        Element::Sand => 2,
        _ => panic!("Element {:?} not supported", element),
    };

    row_index * 3 + column_index
}
