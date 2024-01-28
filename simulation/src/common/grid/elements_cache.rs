use bevy::prelude::*;

use crate::{common::position::Position, nest_simulation::element::Element};

#[derive(Debug)]
pub struct ElementsCache {
    cache: Vec<Vec<Entity>>,
}

impl ElementsCache {
    pub fn new(cache: Vec<Vec<Entity>>) -> Self {
        Self { cache }
    }

    // TODO: These should probably not return references?
    pub fn get_element_entity(&self, position: Position) -> Option<&Entity> {
        self.cache
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
    }

    pub fn element_entity(&self, position: Position) -> &Entity {
        self.get_element_entity(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    pub fn set_element(&mut self, position: Position, entity: Entity) {
        self.cache[position.y as usize][position.x as usize] = entity;
    }

    pub fn is_element(
        &self,
        elements_query: &Query<&Element>,
        position: Position,
        search_element: Element,
    ) -> bool {
        self.get_element_entity(position).map_or(false, |&element| {
            elements_query
                .get(element)
                .map_or(false, |queried_element| *queried_element == search_element)
        })
    }

    // Returns true if every element in `positions` matches the provided Element type.
    // NOTE: This returns true if given 0 positions.
    pub fn is_all_element(
        &self,
        elements_query: &Query<&Element>,
        positions: &[Position],
        search_element: Element,
    ) -> bool {
        positions
            .iter()
            .all(|&position| self.is_element(elements_query, position, search_element))
    }
}
