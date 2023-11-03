use bevy::prelude::*;

use crate::{
    settings::Settings,
    story::{common::position::Position, element::Element},
};

use super::grid::{elements_cache::ElementsCache, Grid};

/// Note the intentional omission of reflection/serialization.
/// This is because Nest is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug)]
pub struct Nest {
    surface_level: isize,
}

impl Nest {
    pub fn new(surface_level: isize) -> Self {
        Self { surface_level }
    }

    pub fn surface_level(&self) -> isize {
        self.surface_level
    }

    pub fn is_aboveground(&self, position: &Position) -> bool {
        !self.is_underground(position)
    }

    pub fn is_underground(&self, position: &Position) -> bool {
        position.y > self.surface_level
    }
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to element entities for quick lookup outside of ECS architecture.
///
/// This is used to speed up most logic because there's a consistent need throughout the application to know what elements are
/// at or near a given position.
pub fn setup_nest(
    element_query: Query<(&mut Position, Entity), With<Element>>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let mut elements_cache = vec![
        vec![Entity::PLACEHOLDER; settings.nest_width as usize];
        settings.nest_height as usize
    ];

    for (position, entity) in element_query.iter() {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    commands.spawn((
        Grid::new(
            settings.nest_width,
            settings.nest_height,
            ElementsCache::new(elements_cache),
        ),
        Nest::new(settings.get_surface_level()),
    ));
}

pub fn teardown_nest(mut commands: Commands, nest_entity_query: Query<Entity, With<Nest>>) {
    let nest_entity = nest_entity_query.single();

    commands.entity(nest_entity).despawn();
}
