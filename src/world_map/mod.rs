use bevy::prelude::*;

use crate::{element::Element, settings::Settings};

pub mod position;
pub mod save;

use self::position::Position;

#[derive(Resource, Debug)]
pub struct WorldMap {
    width: isize,
    height: isize,
    surface_level: isize,
    elements_cache: Vec<Vec<Entity>>,
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to element entities for quick lookup outside of ECS architecture.
///
/// This is used to speed up most logic because there's a consistent need throughout the application to know what elements are
/// at or near a given position.
pub fn setup_world_map(
    element_query: Query<(&mut Position, Entity), With<Element>>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let mut elements_cache = vec![
        vec![Entity::PLACEHOLDER; settings.world_width as usize];
        settings.world_height as usize
    ];

    for (position, entity) in element_query.iter() {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    commands.insert_resource(WorldMap::new(
        settings.world_width,
        settings.world_height,
        settings.get_surface_level(),
        elements_cache,
    ));
}

pub fn teardown_world_map(mut commands: Commands) {
    commands.remove_resource::<WorldMap>();
}

impl WorldMap {
    pub fn new(
        width: isize,
        height: isize,
        surface_level: isize,
        elements_cache: Vec<Vec<Entity>>,
    ) -> Self {
        WorldMap {
            width,
            height,
            surface_level,
            elements_cache,
        }
    }

    pub fn width(&self) -> &isize {
        &self.width
    }

    pub fn height(&self) -> &isize {
        &self.height
    }

    pub fn surface_level(&self) -> &isize {
        &self.surface_level
    }

    pub fn is_aboveground(&self, position: &Position) -> bool {
        !self.is_underground(position)
    }

    pub fn is_underground(&self, position: &Position) -> bool {
        position.y > self.surface_level
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    pub fn get_element(&self, position: Position) -> Option<&Entity> {
        self.elements_cache
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
    }

    pub fn element(&self, position: Position) -> &Entity {
        self.get_element(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    pub fn set_element(&mut self, position: Position, entity: Entity) {
        self.elements_cache[position.y as usize][position.x as usize] = entity;
    }

    pub fn is_element(
        &self,
        elements_query: &Query<&Element>,
        position: Position,
        search_element: Element,
    ) -> bool {
        self.get_element(position).map_or(false, |&element| {
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
