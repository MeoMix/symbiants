mod elements_cache;

use bevy::prelude::*;

use crate::{
    settings::Settings,
    story::{common::position::Position, element::Element},
};

use self::elements_cache::ElementsCache;

/// Note the intentional omission of reflection/serialization.
/// This is because Nest is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug)]
pub struct Nest {
    width: isize,
    height: isize,
    surface_level: isize,
    elements_cache: ElementsCache,
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

    commands.spawn(Nest::new(
        settings.nest_width,
        settings.nest_height,
        settings.get_surface_level(),
        ElementsCache::new(elements_cache),
    ));
}

pub fn teardown_nest(mut commands: Commands, nest_entity_query: Query<Entity, With<Nest>>) {
    let nest_entity = nest_entity_query.single();

    commands.entity(nest_entity).despawn();
}

impl Nest {
    pub fn new(
        width: isize,
        height: isize,
        surface_level: isize,
        elements_cache: ElementsCache,
    ) -> Self {
        Nest {
            width,
            height,
            surface_level,
            elements_cache,
        }
    }

    pub fn width(&self) -> isize {
        self.width
    }

    pub fn height(&self) -> isize {
        self.height
    }

    pub fn surface_level(&self) -> isize {
        self.surface_level
    }

    pub fn elements(&self) -> &ElementsCache {
        &self.elements_cache
    }

    pub fn elements_mut(&mut self) -> &mut ElementsCache {
        &mut self.elements_cache
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

    // TODO: This still isn't the right spot for it I think, but living here for now. Maybe move into a dedicate UI layer later on
    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn as_world_position(&self, position: Position) -> Vec3 {
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
}
