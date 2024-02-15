use std::marker::PhantomData;

use crate::common::{
    pheromone::{Pheromone, PheromoneDuration, PheromoneMap, PheromoneStrength},
    position::Position,
    Zone,
};
use bevy::{ecs::system::Command, prelude::*};

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
        let pheromone_entities = world
            .resource::<PheromoneMap<Z>>()
            .map
            .get(&self.position)
            .map_or_else(Vec::new, |pheromone_entities| pheromone_entities.clone());

        let has_pheromone = pheromone_entities.iter().any(|&entity| {
            world
                .query::<&Pheromone>()
                .get(world, entity)
                .map_or(false, |pheromone| *pheromone == self.pheromone)
        });

        // TODO: Instead of early exit, seems better to "sum" the PheromoneStrengths, but don't want the strength to grow indefinitely.
        if has_pheromone {
            return;
        }

        let pheromone_entity = world
            .spawn((
                self.position,
                self.pheromone,
                PheromoneDuration::default(),
                self.pheromone_strength,
                self.zone,
            ))
            .id();

        let mut pheromone_map = world.resource_mut::<PheromoneMap<Z>>();
        pheromone_map
            .map
            .entry(self.position)
            .and_modify(|entities| entities.push(pheromone_entity))
            .or_insert_with(|| vec![pheromone_entity]);
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
            .resource_mut::<PheromoneMap<Z>>()
            .map
            .get_mut(&self.position)
        {
            if let Some(pos) = pheromone_entities
                .iter()
                .position(|&e| e == self.pheromone_entity)
            {
                pheromone_entities.remove(pos);

                if pheromone_entities.is_empty() {
                    world
                        .resource_mut::<PheromoneMap<Z>>()
                        .map
                        .remove(&self.position);
                }

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
