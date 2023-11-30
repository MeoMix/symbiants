use super::Element;
use crate::story::{
    common::position::Position,
    grid::Grid,
    nest_simulation::nest::{AtNest, Nest},
};
use bevy::prelude::*;

#[derive(Resource)]
pub struct ElementSpriteHandles {
    pub dirt: Vec<Handle<Image>>,
    pub sand: Vec<Handle<Image>>,
    pub food: Vec<Handle<Image>>,
    pub air: Handle<Image>,
}

impl ElementSpriteHandles {
    pub fn get_handles(&self) -> Vec<&Handle<Image>> {
        let handles = [&self.dirt, &self.sand, &self.food];

        handles
            .iter()
            .flat_map(|handle_vec| handle_vec.iter().map(|handle| handle))
            .chain(std::iter::once(&self.air))
            .collect()
    }
}

pub struct ElementExposure {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

pub fn get_element_texture(
    element: &Element,
    index: usize,
    element_sprite_handles: &Res<ElementSpriteHandles>,
) -> Handle<Image> {
    match element {
        // Air is transparent - reveals background color such as tunnel or sky
        Element::Air => element_sprite_handles.air.clone(),
        Element::Dirt => element_sprite_handles.dirt[index].clone(),
        Element::Sand => element_sprite_handles.sand[index].clone(),
        Element::Food => element_sprite_handles.food[index].clone(),
    }
}

// TODO: super hardcoded to the order they appear in sheet.png
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

pub fn on_spawn_element(
    added_elements_query: Query<(Entity, &Position, &Element), Added<Element>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_sprite_handles: Res<ElementSpriteHandles>,
    mut commands: Commands,
) {
    let grid = nest_query.single();

    for (entity, position, element) in &added_elements_query {
        update_element_sprite(
            entity,
            element,
            position,
            &element_sprite_handles,
            &elements_query,
            &grid,
            &mut commands,
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
                        &element_sprite_handles,
                        &elements_query,
                        &grid,
                        &mut commands,
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
    element_sprite_handles: &Res<ElementSpriteHandles>,
    elements_query: &Query<&Element>,
    grid: &Grid,
    commands: &mut Commands,
) {
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

    let element_index = get_element_index(element_exposure);

    // TODO: Need to not overwrite existing visibility - really only want to overwrite these 3 and add the others if they don't exist.

    // BUG: https://github.com/bevyengine/bevy/issues/1949
    // Intentionally not using SpriteSheetBundle due to subpixel rounding causing bleed artifacts.
    commands.entity(element_entity).insert(SpriteBundle {
        texture: get_element_texture(element, element_index, &element_sprite_handles),
        sprite: Sprite {
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        },
        transform: Transform::from_translation(grid.grid_to_world_position(*element_position)),
        // TODO: Maintain existing visibility if set
        ..default()
    });
}

pub fn rerender_elements(
    mut element_query: Query<(&Position, &Element, Entity), With<AtNest>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_sprite_handles: Res<ElementSpriteHandles>,
    mut commands: Commands,
) {
    let grid = nest_query.single();

    for (position, element, entity) in element_query.iter_mut() {
        update_element_sprite(
            entity,
            element,
            position,
            &element_sprite_handles,
            &elements_query,
            &grid,
            &mut commands,
        );
    }
}

pub fn on_update_element_position(
    mut element_query: Query<(&Position, &Element, Entity), Changed<Position>>,
    elements_query: Query<&Element>,
    nest_query: Query<&Grid, With<Nest>>,
    element_sprite_handles: Res<ElementSpriteHandles>,
    mut commands: Commands,
) {
    let grid = nest_query.single();

    for (position, element, entity) in element_query.iter_mut() {
        update_element_sprite(
            entity,
            element,
            position,
            &element_sprite_handles,
            &elements_query,
            &grid,
            &mut commands,
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
                        &element_sprite_handles,
                        &elements_query,
                        &grid,
                        &mut commands,
                    );
                }
            }
        }
    }
}
