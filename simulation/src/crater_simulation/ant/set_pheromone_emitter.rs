use crate::{common::ant::AntInventory, crater_simulation::crater::AtCrater};
use bevy::prelude::*;

use super::emit_pheromone::{LeavingFood, LeavingNest};

pub fn ants_set_pheromone_emitter(
    ants_query: Query<(Entity, Ref<AntInventory>), With<AtCrater>>,
    mut commands: Commands,
) {
    // If an ant recently began carrying food then it should switch to emitting "LeavingFood" pheromone.

    // TODO: if ant arrives from nest it should switch to emitting "LeavingNest" pheromone.

    for (ant_entity, inventory) in ants_query.iter() {
        if inventory.is_changed() && inventory.0.is_some() {
            commands
                .entity(ant_entity)
                // TODO: It would be nice to convey through the type system that only one can be applied at a time
                .insert(LeavingFood(1000))
                .remove::<LeavingNest>();
        }
    }
}
