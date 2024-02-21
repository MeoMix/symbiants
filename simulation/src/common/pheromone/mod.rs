pub mod commands;

use self::commands::PheromoneCommandsExt;
use super::{position::Position, Zone};
use crate::story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_HOUR};
use bevy::{prelude::*, utils::HashMap};
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

// #[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect)]
// #[reflect(Component)]
// pub struct PheromoneDuration {
//     value: f32,
//     max: f32,
// }

// impl Default for PheromoneDuration {
//     fn default() -> Self {
//         Self {
//             value: 0.0,
//             max: 100.0,
//         }
//     }
// }

// impl PheromoneDuration {
//     pub fn max(&self) -> f32 {
//         self.max
//     }

//     pub fn tick(&mut self, rate_of_pheromone_expiration: f32) {
//         self.value = (self.value + rate_of_pheromone_expiration).min(self.max);
//     }

//     pub fn is_expired(&self) -> bool {
//         self.value >= self.max / 2.0
//     }
// }

/// Note the intentional omission of reflection/serialization.
/// This is because PheromoneMap is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Resource, Debug)]
pub struct PheromoneMap<Z: Zone> {
    map: HashMap<Position, Vec<Entity>>,
    _marker: PhantomData<Z>,
}

impl<Z: Zone> PheromoneMap<Z> {
    pub fn new(map: HashMap<Position, Vec<Entity>>) -> Self {
        Self {
            map,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, position: &Position) -> Option<&Vec<Entity>> {
        self.map.get(position)
    }
}

/// Note the intentional omission of PheromoneMap. It would be wasteful to persist
/// because it's able to be trivially regenerated at runtime.
pub fn register_pheromone(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Pheromone>();
    app_type_registry.write().register::<PheromoneStrength>();
    // app_type_registry.write().register::<PheromoneDuration>();
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
    let mut pheromone_map: HashMap<Position, Vec<Entity>> = HashMap::new();

    for (position, entity) in pheromone_query.iter() {
        pheromone_map.entry(*position).or_insert_with(Vec::new).push(entity);
    }

    commands.insert_resource(PheromoneMap::<Z>::new(pheromone_map));
}

pub fn remove_pheromone_resources<Z: Zone>(mut commands: Commands) {
    commands.remove_resource::<PheromoneMap<Z>>();
}

pub fn pheromone_duration_tick<Z: Zone>(
    mut pheromone_query: Query<(&mut PheromoneStrength, &Position, Entity), With<Z>>,
    mut commands: Commands,
) {
    for (mut pheromone_strength, position, pheromone_entity) in pheromone_query.iter_mut() {
        // Get 100% expired once every hour
        // let rate_of_pheromone_expiration =
        //     pheromone_duration.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;

        // pheromone_duration.tick(rate_of_pheromone_expiration);
        let increment = pheromone_strength.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;
        pheromone_strength.increment(-increment);

        if pheromone_strength.value() <= 0.0 {
            commands.despawn_pheromone(pheromone_entity, *position, PhantomData::<Z>);
        }
    }
}
