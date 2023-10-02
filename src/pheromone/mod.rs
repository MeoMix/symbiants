use bevy::{prelude::*, utils::HashMap};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

use crate::{common::register, world_map::position::Position};

pub mod commands;
pub mod ui;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum Pheromone {
    #[default]
    Tunnel,
    Chamber,
}

/// Note the intentional omission of reflection/serialization.
/// This is because PheromoneMap is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Resource, Debug)]
pub struct PheromoneMap(pub HashMap<Position, Entity>);

/// Note the intentional omission of PheromoneMap. It would be wasteful to persist
/// because it's able to be trivially regenerated at runtime.
pub fn register_pheromone(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Pheromone>(&app_type_registry, &mut saveable_registry);
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to pheromone entities for quick lookup outside of ECS architecture.
///
/// This isn't super necessary. Performance impact of O(N) lookup on Pheromone is likely to be negligible.
/// Still, it seemed like a good idea architecturally to have O(1) lookup when Position is known.
pub fn setup_pheromone(
    pheromone_query: Query<(&mut Position, Entity), With<Pheromone>>,
    mut commands: Commands,
) {
    let pheromone_map = pheromone_query
        .iter()
        .map(|(position, entity)| (*position, entity))
        .collect::<HashMap<_, _>>();

    commands.insert_resource(PheromoneMap(pheromone_map));
}
