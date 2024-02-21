use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        ant::Initiative,
        pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

// TODO: Need to persist LeavingNest and LeavingFood
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct LeavingNest(pub f32);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct LeavingFood(pub f32);

pub fn ants_emit_pheromone(
    mut ants_query: Query<
        (
            Entity,
            &Position,
            &Initiative,
            AnyOf<(&mut LeavingFood, &mut LeavingNest)>,
        ),
        With<AtCrater>,
    >,
    mut commands: Commands,
) {
    for (ant_entity, position, initiative, (leaving_food, leaving_nest)) in ants_query.iter_mut() {
        // Ants don't move every tick, if initative isn't checked then will leave multiple pheromone entries on same tile
        if !initiative.can_move() {
            continue;
        }

        if let Some(mut leaving_food) = leaving_food {
            commands.spawn_pheromone(
                *position,
                Pheromone::Food,
                // TODO: Read 100 from config
                PheromoneStrength::new(leaving_food.0, 1000.0),
                AtCrater,
            );

            leaving_food.0 -= 1.0;

            if leaving_food.0 <= 0.0 {
                commands.entity(ant_entity).remove::<LeavingFood>();
            }
        }

        if let Some(mut leaving_nest) = leaving_nest {
            commands.spawn_pheromone(
                *position,
                Pheromone::Nest,
                PheromoneStrength::new(leaving_nest.0, 1000.0),
                AtCrater,
            );

            leaving_nest.0 -= 1.0;

            if leaving_nest.0 <= 0.0 {
                commands.entity(ant_entity).remove::<LeavingNest>();
            }
        }
    }
}
