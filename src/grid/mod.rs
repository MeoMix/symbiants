use bevy::prelude::*;
use std::ops::Add;

use crate::{element::Element, settings::Settings};

pub mod position;
pub mod save;

use chrono::{DateTime, Utc};

use self::position::Position;

#[derive(Resource, Debug)]
pub struct WorldMap {
    width: isize,
    height: isize,
    surface_level: isize,
    created_at: DateTime<Utc>,
    elements_cache: Vec<Vec<Entity>>,
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to element entities for quick lookup outside of ECS architecture.
pub fn setup_caches(world: &mut World) {
    let (width, height, surface_level) = {
        let settings = world.resource::<Settings>();
        (
            settings.world_width,
            settings.world_height,
            settings.get_surface_level(),
        )
    };

    let elements_cache = create_elements_cache(world, width, height);
    world.insert_resource(WorldMap::new(width, height, surface_level, elements_cache));
}

pub fn cleanup_caches(world: &mut World) {
    world.remove_resource::<WorldMap>();
}

// Create a cache which allows for spatial querying of Elements. This is used to speed up
// most logic because there's a consistent need throughout the application to know what elements are
// at or near a given position.
fn create_elements_cache(world: &mut World, width: isize, height: isize) -> Vec<Vec<Entity>> {
    let mut elements_cache = vec![vec![Entity::PLACEHOLDER; width as usize]; height as usize];

    for (position, entity) in world
        .query_filtered::<(&mut Position, Entity), With<Element>>()
        .iter(&world)
    {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    elements_cache
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
            created_at: Utc::now(),
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

    // round up so start at 1
    pub fn days_old(&self) -> i64 {
        let now = Utc::now();
        let duration = now - self.created_at;
        duration.num_days().add(1)
    }

    pub fn is_below_surface(&self, position: &Position) -> bool {
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
