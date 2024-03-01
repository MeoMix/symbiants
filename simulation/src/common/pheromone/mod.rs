pub mod commands;

use self::commands::PheromoneCommandsExt;
use super::{position::Position, Zone};
use crate::story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_HOUR};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// TODO: It's weird that Pheromone defaults to Tunnel when, in reality, no default would be more sensible.
/// TODO: It's possible that Pheromone should be split in two: CraterPheromone and NestPheromone. There's no overlap between the two.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum Pheromone {
    #[default]
    Tunnel,
    Chamber,
    Food,
    Nest,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct PheromoneStrength {
    value: f32,
    max: f32,
}

impl PheromoneStrength {
    pub fn new(value: f32, max: f32) -> Self {
        if value > max {
            panic!("PheromoneStrength value cannot be greater than max");
        }

        Self { value, max }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn increment(&mut self, value: f32) {
        self.value = (self.value + value).min(self.max);
    }
}

/// Note the intentional omission of reflection/serialization.
/// This is because PheromoneMap is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Resource, Debug)]
pub struct PheromoneEntityPositionCache<Z: Zone> {
    // 2D vector, sparsely populated, multiple pheromones per position
    // Use a 2D vec over a HashMap for performance
    cache: Vec<Vec<Vec<Entity>>>,
    _marker: PhantomData<Z>,
}

impl<Z: Zone> PheromoneEntityPositionCache<Z> {
    pub fn new(cache: Vec<Vec<Vec<Entity>>>) -> Self {
        Self {
            cache,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, position: &Position) -> Option<&Vec<Entity>> {
        let (x, y) = (position.x as usize, position.y as usize);

        if x < self.cache.len() {
            if y < self.cache[x].len() {
                Some(&self.cache[x][y])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, position: &Position) -> Option<&mut Vec<Entity>> {
        let (x, y) = (position.x as usize, position.y as usize);

        if x < self.cache.len() {
            if y < self.cache[x].len() {
                Some(&mut self.cache[x][y])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn add_or_update_entity(&mut self, position: &Position, pheromone_entity: Entity) {
        let (x, y) = (position.x as usize, position.y as usize);

        // Ensure the outer vector is large enough to include x
        if x >= self.cache.len() {
            self.cache.resize(x + 1, Vec::new());
        }

        // Ensure the inner vector at self.cache[x] is large enough to include y
        if y >= self.cache[x].len() {
            self.cache[x].resize(y + 1, Vec::new());
        }

        // Now that we've ensured the vectors are properly sized, we can directly add the entity
        self.cache[x][y].push(pheromone_entity);
    }
}

/// Note the intentional omission of PheromoneMap. It would be wasteful to persist
/// because it's able to be trivially regenerated at runtime.
pub fn register_pheromone(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Pheromone>();
    app_type_registry.write().register::<PheromoneStrength>();
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to pheromone entities for quick lookup outside of ECS architecture.
///
/// This isn't super necessary. Performance impact of O(N) lookup on Pheromone is likely to be negligible.
/// Still, it seemed like a good idea architecturally to have O(1) lookup when Position is known.
pub fn initialize_pheromone_resources<Z: Zone>(
    pheromone_query: Query<(&mut Position, Entity), (With<Pheromone>, With<Z>)>,
    mut commands: Commands,
) {
    let mut pheromone_cache: Vec<Vec<Vec<Entity>>> = Vec::new();

    for (position, entity) in pheromone_query.iter() {
        let (x, y) = (position.x as usize, position.y as usize);
    
        if x >= pheromone_cache.len() {
            pheromone_cache.resize(x + 1, Vec::new());
        }
    
        if y >= pheromone_cache[x].len() {
            pheromone_cache[x].resize(y + 1, Vec::new());
        }
    
        pheromone_cache[x][y].push(entity);
    }

    commands.insert_resource(PheromoneEntityPositionCache::<Z>::new(pheromone_cache));
}

pub fn remove_pheromone_resources<Z: Zone>(mut commands: Commands) {
    commands.remove_resource::<PheromoneEntityPositionCache<Z>>();
}

pub fn decay_pheromone_strength<Z: Zone>(
    mut pheromone_query: Query<(&mut PheromoneStrength, &Position, Entity), With<Z>>,
    mut commands: Commands,
) {
    for (mut pheromone_strength, position, pheromone_entity) in pheromone_query.iter_mut() {
        // 100% expired once every hour
        let increment = pheromone_strength.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;
        pheromone_strength.increment(-increment);

        if pheromone_strength.value() <= 0.0 {
            commands.despawn_pheromone(pheromone_entity, *position, PhantomData::<Z>);
        }
    }
}
