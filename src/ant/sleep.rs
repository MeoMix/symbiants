use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use crate::{
    story_time::StoryElapsedTicks,
    world_map::{position::Position, WorldMap},
};

use super::{AntOrientation, AntRole, Initiative};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Asleep;

pub fn ants_sleep(
    ants_query: Query<(Entity, &AntRole, &Position, &AntOrientation), With<Initiative>>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    story_elapsed_ticks: Res<StoryElapsedTicks>,
) {
    if !story_elapsed_ticks.is_nighttime() {
        return;
    }

    for (ant_entity, ant_role, ant_position, ant_orientation) in ants_query.iter() {
        // If ant is a worker, underground, horizontal, and its night time, then add the sleep component to them.
        if *ant_role == AntRole::Queen {
            continue;
        }

        if world_map.is_aboveground(ant_position) {
            continue;
        }

        if !ant_orientation.is_rightside_up() {
            continue;
        }

        commands
            .entity(ant_entity)
            .insert(Asleep)
            .remove::<Initiative>();
    }
}

pub fn ants_wake(
    ants_query: Query<Entity, With<Asleep>>,
    mut commands: Commands,
    story_elapsed_ticks: Res<StoryElapsedTicks>,
    mut rng: ResMut<GlobalRng>,
) {
    if story_elapsed_ticks.is_nighttime() {
        return;
    }

    for ant_entity in ants_query.iter() {
        commands
            .entity(ant_entity)
            .remove::<Asleep>()
            .insert(Initiative::new(&mut rng.reborrow()));
    }
}
