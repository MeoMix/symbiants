use super::Element;
use crate::{
    simulation::SpriteSheets,
    world_map::{position::Position, WorldMap},
};
use bevy::prelude::*;

pub struct ElementExposure {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
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
    world_map: Res<WorldMap>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    for (entity, position, element) in &added_elements_query {
        if *element != Element::Air {
            update_element_sprite(
                entity,
                element,
                position,
                &sprite_sheets,
                &elements_query,
                &world_map,
                &mut commands,
            );
        }

        let adjacent_positions = position.get_adjacent_positions();

        for adjacent_position in adjacent_positions {
            if let Some(adjacent_element_entity) = world_map.get_element_entity(adjacent_position) {
                let adjacent_element = elements_query.get(*adjacent_element_entity).unwrap();

                if *adjacent_element != Element::Air {
                    update_element_sprite(
                        *adjacent_element_entity,
                        adjacent_element,
                        &adjacent_position,
                        &sprite_sheets,
                        &elements_query,
                        &world_map,
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
    sprite_sheets: &Res<SpriteSheets>,
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
    commands: &mut Commands,
) {
    let element_exposure = ElementExposure {
        north: world_map.is_element(
            &elements_query,
            *element_position - Position::Y,
            Element::Air,
        ),
        east: world_map.is_element(
            &elements_query,
            *element_position + Position::X,
            Element::Air,
        ),
        south: world_map.is_element(
            &elements_query,
            *element_position + Position::Y,
            Element::Air,
        ),
        west: world_map.is_element(
            &elements_query,
            *element_position - Position::X,
            Element::Air,
        ),
    };

    let mut sprite = TextureAtlasSprite::new(get_element_index(element_exposure));
    sprite.custom_size = Some(Vec2::splat(1.0));

    let texture_atlas = match element {
        Element::Air => panic!("Air element should not be rendered"),
        Element::Dirt => sprite_sheets.dirt.clone(),
        Element::Food => sprite_sheets.food.clone(),
        Element::Sand => sprite_sheets.sand.clone(),
    };

    commands.entity(element_entity).insert(SpriteSheetBundle {
        sprite,
        texture_atlas,
        transform: Transform::from_translation(element_position.as_world_position(&world_map)),
        ..default()
    });
}

pub fn rerender_elements(
    mut element_query: Query<(&Position, Option<&mut Transform>, &Element, Entity)>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    for (position, transform, element, entity) in element_query.iter_mut() {
        // TODO: Air doesn't have a transform
        if let Some(mut transform) = transform {
            transform.translation = position.as_world_position(&world_map);
        }

        if *element != Element::Air {
            update_element_sprite(
                entity,
                element,
                position,
                &sprite_sheets,
                &elements_query,
                &world_map,
                &mut commands,
            );
        }

        let adjacent_positions = position.get_adjacent_positions();

        for adjacent_position in adjacent_positions {
            if let Some(adjacent_element_entity) = world_map.get_element_entity(adjacent_position) {
                let adjacent_element = elements_query.get(*adjacent_element_entity).unwrap();

                if *adjacent_element != Element::Air {
                    update_element_sprite(
                        *adjacent_element_entity,
                        adjacent_element,
                        &adjacent_position,
                        &sprite_sheets,
                        &elements_query,
                        &world_map,
                        &mut commands,
                    );
                }
            }
        }
    }
}

pub fn on_update_element_position(
    mut element_query: Query<
        (&Position, Option<&mut Transform>, &Element, Entity),
        Changed<Position>,
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    sprite_sheets: Res<SpriteSheets>,
    mut commands: Commands,
) {
    for (position, transform, element, entity) in element_query.iter_mut() {
        // TODO: Air doesn't have a transform
        if let Some(mut transform) = transform {
            transform.translation = position.as_world_position(&world_map);
        }

        if *element != Element::Air {
            update_element_sprite(
                entity,
                element,
                position,
                &sprite_sheets,
                &elements_query,
                &world_map,
                &mut commands,
            );
        }

        let adjacent_positions = position.get_adjacent_positions();

        for adjacent_position in adjacent_positions {
            if let Some(adjacent_element_entity) = world_map.get_element_entity(adjacent_position) {
                let adjacent_element = elements_query.get(*adjacent_element_entity).unwrap();

                if *adjacent_element != Element::Air {
                    update_element_sprite(
                        *adjacent_element_entity,
                        adjacent_element,
                        &adjacent_position,
                        &sprite_sheets,
                        &elements_query,
                        &world_map,
                        &mut commands,
                    );
                }
            }
        }
    }
}
