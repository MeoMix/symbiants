use super::Element;
use crate::{
    app_state::AppState,
    story::{
        common::position::Position,
        grid::Grid,
        nest_simulation::nest::{AtNest, Nest},
    },
};
use bevy::{asset::LoadState, prelude::*, utils::HashMap};

pub fn on_spawn_element(
    added_elements_query: Query<(Entity, &Position, &Element), Added<Element>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    element_texture_atlas_handles: Res<ElementTextureAtlasHandles>,
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
            &element_texture_atlas_handles,
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
                        &element_texture_atlas_handles,
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
    element_texture_atlas_handles: &Res<ElementTextureAtlasHandles>,
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

    let mut sprite = TextureAtlasSprite::new(get_element_index(element_exposure));
    sprite.custom_size = Some(Vec2::splat(1.0));

    commands.entity(element_entity).insert(SpriteSheetBundle {
        sprite,
        texture_atlas: element_texture_atlas_handles.get_handle(element).clone(),
        transform: Transform::from_translation(grid.grid_to_world_position(*element_position)),
        // TODO: Maintain existing visibility if set?
        ..default()
    });
}

pub fn rerender_elements(
    mut element_query: Query<(&Position, &Element, Entity), With<AtNest>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    element_texture_atlas_handles: Res<ElementTextureAtlasHandles>,
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
            &element_texture_atlas_handles,
        );
    }
}

pub fn on_update_element_position(
    mut element_query: Query<(&Position, &Element, Entity), Changed<Position>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    mut commands: Commands,
    element_texture_atlas_handles: Res<ElementTextureAtlasHandles>,
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
            &element_texture_atlas_handles,
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
                        &element_texture_atlas_handles,
                    );
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ElementSpriteSheetHandles(HashMap<Element, Handle<Image>>);

impl ElementSpriteSheetHandles {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Element, &Handle<Image>)> {
        self.0.iter()
    }

    pub fn insert(&mut self, element: Element, handle: Handle<Image>) {
        self.0.insert(element, handle);
    }

    pub fn get_all(&self) -> Vec<&Handle<Image>> {
        self.0.values().collect()
    }
}

#[derive(Resource)]
pub struct ElementTextureAtlasHandles(HashMap<Element, Handle<TextureAtlas>>);

impl ElementTextureAtlasHandles {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, element: Element, handle: Handle<TextureAtlas>) {
        if element == Element::Air {
            panic!("Air element should not be rendered");
        }

        self.0.insert(element, handle);
    }

    pub fn get_handle(&self, element: &Element) -> &Handle<TextureAtlas> {
        if element == &Element::Air {
            panic!("Air element should not be rendered");
        }

        self.0
            .get(element)
            .expect("Texture atlas not found for element")
    }
}

pub fn start_load_element_sprite_sheets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let mut sprite_sheet_handles = ElementSpriteSheetHandles::new();

    for element in [Element::Dirt, Element::Food, Element::Sand] {
        let element_str = format!("{:?}", element).to_lowercase();
        let path = format!("textures/element/{}/sprite_sheet.png", element_str);
        let handle = asset_server.load::<Image>(&path);
        sprite_sheet_handles.insert(element, handle);
    }

    commands.insert_resource(sprite_sheet_handles);
}

pub fn check_element_sprite_sheets_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    element_sprite_sheet_handles: Res<ElementSpriteSheetHandles>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let loaded = element_sprite_sheet_handles
        .get_all()
        .iter()
        .all(|&image_handle| asset_server.load_state(image_handle) == LoadState::Loaded);

    if loaded {
        let mut atlas_handles = ElementTextureAtlasHandles::new();

        for (element, image_handle) in element_sprite_sheet_handles.iter() {
            let texture_atlas = create_texture_atlas(image_handle.clone());
            let atlas_handle = texture_atlases.add(texture_atlas);
            atlas_handles.insert(*element, atlas_handle);
        }

        commands.insert_resource(atlas_handles);

        next_state.set(AppState::TryLoadSave);
    }
}

/// Create a texture atlas from a sprite sheet image.
/// BUG: https://github.com/bevyengine/bevy/issues/1949
/// Need to use too small tile size + padding to prevent bleeding into adjacent sprites on sheet.
fn create_texture_atlas(texture_handle: Handle<Image>) -> TextureAtlas {
    TextureAtlas::from_grid(
        texture_handle,
        Vec2::splat(120.0),
        1,
        16,
        Some(Vec2::splat(8.0)),
        Some(Vec2::splat(4.0)),
    )
}

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
pub fn get_element_index(exposure: ElementExposure) -> usize {
    match exposure {
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
    }
}
