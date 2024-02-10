use super::crater::AtCrater;
use crate::{
    common::position::Position,
    nest_simulation::pheromone::{
        commands::PheromoneCommandsExt, Pheromone, PheromoneDuration, PheromoneMap,
    },
    story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_HOUR},
};
use bevy::{prelude::*, utils::HashMap};

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to pheromone entities for quick lookup outside of ECS architecture.
///
/// This isn't super necessary. Performance impact of O(N) lookup on Pheromone is likely to be negligible.
/// Still, it seemed like a good idea architecturally to have O(1) lookup when Position is known.
pub fn initialize_pheromone_resources(
    pheromone_query: Query<(&mut Position, Entity), With<Pheromone>>,
    mut commands: Commands,
) {
    let pheromone_map = pheromone_query
        .iter()
        .map(|(position, entity)| (*position, entity))
        .collect::<HashMap<_, _>>();

    commands.insert_resource(PheromoneMap::<AtCrater>::new(pheromone_map));
}

pub fn remove_pheromone_resources(mut commands: Commands) {
    commands.remove_resource::<PheromoneMap<AtCrater>>();
}

pub fn pheromone_duration_tick(
    mut pheromone_query: Query<(&mut PheromoneDuration, &Position, Entity), With<AtCrater>>,
    mut commands: Commands,
) {
    for (mut pheromone_duration, position, pheromone_entity) in pheromone_query.iter_mut() {
        // Get 100% expired once every hour
        let rate_of_pheromone_expiration =
            pheromone_duration.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;

        pheromone_duration.tick(rate_of_pheromone_expiration);

        if pheromone_duration.is_expired() {
            commands.despawn_pheromone(pheromone_entity, *position, AtCrater);
        }
    }
}
