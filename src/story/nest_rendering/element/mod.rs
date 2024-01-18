use crate::{
    app_state::AppState,
    story::{
        common::position::Position,
        element::{Air, Element, ElementExposure},
        grid::Grid,
        nest_rendering::common::{ModelViewEntityMap, VisibleGrid},
        nest_simulation::nest::{AtNest, Nest},
    },
};
use bevy::{asset::LoadState, prelude::*, utils::hashbrown::HashSet};
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
pub struct ElementTilemap;

// When an element's position or exposure changes - redraw its sprite.
// This will run when an element is first spawned because Changed is true on create.
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

pub fn on_despawn_element(
    mut removed: RemovedComponents<Element>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let model_entities = &mut removed.read().collect::<HashSet<_>>();

    for model_entity in model_entities.iter() {
        if let Some(view_entity) = model_view_entity_map.0.remove(model_entity) {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}

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
    if let Some(&element_view_entity) = model_view_entity_map.0.get(&element_model_entity) {
        commands.entity(element_view_entity).insert(tile_bundle);
        tile_storage.set(&tile_pos, element_view_entity);
    } else {
        let element_view_entity = commands.spawn(tile_bundle).id();
        model_view_entity_map
            .0
            .insert(element_model_entity, element_view_entity);
        tile_storage.set(&tile_pos, element_view_entity);
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

        next_state.set(AppState::TryLoadSave);
    }
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

pub fn teardown_element(
    mut commands: Commands,
    element_model_query: Query<Entity, With<Element>>,
    element_tilemap_query: Query<Entity, With<ElementTilemap>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    for element_model_entity in element_model_query.iter() {
        if let Some(&element_view_entity) = model_view_entity_map.0.get(&element_model_entity) {
            commands.entity(element_view_entity).despawn_recursive();
            model_view_entity_map.0.remove(&element_model_entity);
        }
    }

    let element_tilemap_entity = element_tilemap_query.single();
    commands.entity(element_tilemap_entity).despawn_recursive();

    // TODO: remove ElementTextureAtlasHandle and ElementSpriteSheetHandle if committing to the full cleanup process
}
