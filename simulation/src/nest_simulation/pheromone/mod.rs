pub mod commands;

use self::commands::PheromoneCommandsExt;
use crate::{
    common::{pheromone::PheromoneDuration, position::Position},
    nest_simulation::nest::AtNest,
    story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_HOUR},
};
use bevy::prelude::*;

// TODO: Probably move this to common?
pub fn pheromone_duration_tick(
    mut pheromone_query: Query<(&mut PheromoneDuration, &Position, Entity), With<AtNest>>,
    mut commands: Commands,
) {
    for (mut pheromone_duration, position, pheromone_entity) in pheromone_query.iter_mut() {
        // Get 100% expired once every hour
        let rate_of_pheromone_expiration =
            pheromone_duration.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;

        pheromone_duration.tick(rate_of_pheromone_expiration);

        if pheromone_duration.is_expired() {
            commands.despawn_pheromone(pheromone_entity, *position, AtNest);
        }
    }
}
