use std::marker::PhantomData;

use crate::common::{
    pheromone::{Pheromone, PheromoneEntityPositionCache, PheromoneStrength},
    position::Position,
    Zone,
};
use bevy::{ecs::world::Command, prelude::*};

pub trait PheromoneCommandsExt {
    fn spawn_pheromone<Z: Zone>(
        &mut self,
        position: Position,
        pheromone: Pheromone,
        pheromone_strength: PheromoneStrength,
        zone: Z,
    );
    fn despawn_pheromone<Z: Zone>(
        &mut self,
        pheromone_entity: Entity,
        position: Position,
        zone: PhantomData<Z>,
    );
}

impl<'w, 's> PheromoneCommandsExt for Commands<'w, 's> {
    fn spawn_pheromone<Z: Zone>(
        &mut self,
        position: Position,
        pheromone: Pheromone,
        pheromone_strength: PheromoneStrength,
        zone: Z,
    ) {
        self.add(SpawnPheromoneCommand {
            position,
            pheromone,
            pheromone_strength,
            zone,
        })
    }

    fn despawn_pheromone<Z: Zone>(
        &mut self,
        pheromone_entity: Entity,
        position: Position,
        zone: PhantomData<Z>,
    ) {
        self.add(DespawnPheromoneCommand {
            pheromone_entity,
            position,
            zone,
        })
    }
}

struct SpawnPheromoneCommand<Z: Zone> {
    position: Position,
    pheromone: Pheromone,
    pheromone_strength: PheromoneStrength,
    zone: Z,
}

impl<Z: Zone> Command for SpawnPheromoneCommand<Z> {
    /// Spawn a new Pheromone entity and update the associate PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        let default_vec = Vec::new();

        let pheromone_entities = world
            .resource::<PheromoneEntityPositionCache<Z>>()
            .get(&self.position)
            .unwrap_or(&default_vec);

        let matching_pheromone_entity = pheromone_entities.iter().find(|&entity| {
            world
                .entity(*entity)
                .get::<Pheromone>()
                .map_or(false, |pheromone| *pheromone == self.pheromone)
        });

        if let Some(matching_pheromone_entity) = matching_pheromone_entity {
            // Update the existing entity's PheromoneStrength.
            let mut entity = world.entity_mut(*matching_pheromone_entity);
            let mut pheromone_strength = entity.get_mut::<PheromoneStrength>().unwrap();

            pheromone_strength.increment(self.pheromone_strength.value() * 0.1);
            return;
        }

        // Otherwise, insert new pheromone
        let pheromone_entity = world
            .spawn((
                self.position,
                self.pheromone,
                self.pheromone_strength,
                self.zone,
            ))
            .id();

        let mut pheromone_map = world.resource_mut::<PheromoneEntityPositionCache<Z>>();
        pheromone_map.add_or_update_entity(&self.position, pheromone_entity);
    }
}

struct DespawnPheromoneCommand<Z: Zone> {
    pheromone_entity: Entity,
    position: Position,
    zone: PhantomData<Z>,
}

impl<Z: Zone> Command for DespawnPheromoneCommand<Z> {
    /// Despawn a specific Pheromone entity and update the associated PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        if let Some(pheromone_entities) = world
            .resource_mut::<PheromoneEntityPositionCache<Z>>()
            .get_mut(&self.position)
        {
            if let Some(pos) = pheromone_entities
                .iter()
                .position(|&e| e == self.pheromone_entity)
            {
                pheromone_entities.remove(pos);
                world.despawn(self.pheromone_entity);
                return;
            }
        }

        // Log information if the pheromone entity does not exist
        info!(
            "Expected pheromone_entity {:?} at position {:?} to exist",
            self.pheromone_entity, self.position
        );
    }
}
