use super::Zone;
use crate::{common::position::Position, nest_simulation::element::Element};
use bevy::{ecs::system::SystemParam, prelude::*};

/// Note the intentional omission of reflection/serialization.
/// This is because Grid is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug)]
pub struct Grid {
    width: isize,
    height: isize,
    elements_cache: Vec<Vec<Entity>>,
}

impl Grid {
    pub fn new(width: isize, height: isize, elements_cache: Vec<Vec<Entity>>) -> Self {
        Self {
            width,
            height,
            elements_cache,
        }
    }

    pub fn width(&self) -> isize {
        self.width
    }

    pub fn height(&self) -> isize {
        self.height
    }

    // TODO: Feel like these should be eliminated in favor of GridElements and GridElementsMut
    pub fn elements(&self) -> &Vec<Vec<Entity>> {
        &self.elements_cache
    }

    pub fn elements_mut(&mut self) -> &mut Vec<Vec<Entity>> {
        &mut self.elements_cache
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    // TODO: This still isn't the right spot for it I think, but living here for now. Maybe move into a dedicate UI layer later on
    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn grid_to_world_position(&self, position: Position) -> Vec3 {
        let y_offset = self.height as f32 / 2.0;
        let x_offset = self.width as f32 / 2.0;

        Vec3 {
            // NOTE: unit width is 1.0 so add 0.5 to center the position
            x: position.x as f32 - x_offset + 0.5,
            // The view of the model position is just an inversion along the y-axis.
            y: -position.y as f32 + y_offset - 0.5,
            z: 1.0,
        }
    }

    pub fn world_to_grid_position(&self, world_position: Vec2) -> Position {
        let x = world_position.x + (self.width() as f32 / 2.0) - 0.5;
        let y = -world_position.y + (self.height() as f32 / 2.0) - 0.5;

        Position {
            x: x.abs().round() as isize,
            y: y.abs().round() as isize,
        }
    }
}

#[derive(SystemParam)]
pub struct GridElements<'w, 's, Z: Zone> {
    grid: Query<'w, 's, &'static Grid, With<Z>>,
    elements: Query<'w, 's, &'static Element, With<Z>>,
}

// TODO: The interface here is a little unclear - sometimes querying by Entity other times by Position
impl<'w, 's, Z: Zone> GridElements<'w, 's, Z> {
    // TODO: Maybe this shouldn't return a reference?
    pub fn entity(&self, position: Position) -> &Entity {
        self.get_entity(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    // TODO: Maybe this shouldn't return a reference?
    pub fn get_entity(&self, position: Position) -> Option<&Entity> {
        self.grid
            .single()
            .elements()
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
    grid: Query<'w, 's, &'static mut Grid, With<Z>>,
}

impl<'w, 's, Z: Zone> GridElementsMut<'w, 's, Z> {
    pub fn set(&mut self, position: Position, entity: Entity) {
        self.grid.single_mut().elements_mut()[position.y as usize][position.x as usize] = entity;
    }
}
