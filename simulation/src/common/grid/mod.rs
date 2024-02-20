use super::Zone;
use crate::common::{element::Element, position::Position};
use bevy::{ecs::system::SystemParam, prelude::*};

// TODO: Either this should be a Resource or PheromoneMap should be a Component
#[derive(Component, Debug)]
pub struct ElementEntityPositionCache(pub Vec<Vec<Entity>>);

#[derive(Component, Debug)]
pub struct Grid {
    width: isize,
    height: isize,
}

impl Grid {
    pub fn new(width: isize, height: isize) -> Self {
        Self {
            width,
            height,
        }
    }

    pub fn width(&self) -> isize {
        self.width
    }

    pub fn height(&self) -> isize {
        self.height
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }
}

#[derive(SystemParam)]
pub struct GridElements<'w, 's, Z: Zone> {
    elements_cache: Query<'w, 's, &'static ElementEntityPositionCache, With<Z>>,
    elements: Query<'w, 's, &'static Element, With<Z>>,
}

// TODO: The interface here is a little unclear - sometimes querying by Entity other times by Position
impl<'w, 's, Z: Zone> GridElements<'w, 's, Z> {
    pub fn entity(&self, position: Position) -> &Entity {
        self.get_entity(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    pub fn get_entity(&self, position: Position) -> Option<&Entity> {
        self.elements_cache
            .single()
            .0
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
    }

    pub fn element(&self, entity: Entity) -> &Element {
        self.get_element(entity)
            .expect(&format!("Element not found for the entity: {:?}", entity))
    }

    pub fn get_element(&self, entity: Entity) -> Option<&Element> {
        match self.elements.get(entity) {
            Ok(element) => Some(element),
            Err(_) => None,
        }
    }

    pub fn is(&self, position: Position, element: Element) -> bool {
        self.get_entity(position).map_or(false, |&element_entity| {
            self.get_element(element_entity)
                .map_or(false, |&queried_element| queried_element == element)
        })
    }

    // Returns true if every element in `positions` matches the provided Element type.
    // NOTE: This returns true if given 0 positions.
    pub fn is_all(&self, positions: &[Position], element: Element) -> bool {
        positions.iter().all(|&position| self.is(position, element))
    }
}

#[derive(SystemParam)]
pub struct GridElementsMut<'w, 's, Z: Zone> {
    elements_cache: Query<'w, 's, &'static mut ElementEntityPositionCache, With<Z>>,
}

impl<'w, 's, Z: Zone> GridElementsMut<'w, 's, Z> {
    pub fn set(&mut self, position: Position, entity: Entity) {
        self.elements_cache.single_mut().0[position.y as usize][position.x as usize] = entity;
    }
}
