use crate::common::{position::Position, Zone};
use bevy::{ecs::system::Command, prelude::*};

use super::{Pheromone, PheromoneDuration, PheromoneMap, PheromoneStrength};

pub trait PheromoneCommandsExt {
    fn spawn_pheromone<Z: Zone>(
        &mut self,
        position: Position,
        pheromone: Pheromone,
        pheromone_strength: PheromoneStrength,
        zone: Z,
    );
    fn despawn_pheromone<Z: Zone>(&mut self, pheromone_entity: Entity, position: Position, zone: Z);
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
        zone: Z,
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
        // TODO: maybe overwrite existing pheromone instead of noop?
        if let Some(_) = world.resource::<PheromoneMap<Z>>().map.get(&self.position) {
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

        world
            .resource_mut::<PheromoneMap<Z>>()
            .map
            .insert(self.position, pheromone_entity);
    }
}

struct DespawnPheromoneCommand<Z: Zone> {
    pheromone_entity: Entity,
    position: Position,
    zone: Z,
}

impl<Z: Zone> Command for DespawnPheromoneCommand<Z> {
    /// Spawn a new Pheromone entity and update the associate PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        if let Some(pheromone_entity) = world.resource::<PheromoneMap<Z>>().map.get(&self.position) {
            if *pheromone_entity != self.pheromone_entity {
                error!(
                    "Found pheromone_entity {:?}, expected {:?} at position {:?}",
                    pheromone_entity, self.pheromone_entity, self.position
                );
                return;
            }
        } else {
            info!(
                "Expected pheromone_entity {:?} at position {:?} to exist",
                self.pheromone_entity, self.position
            );
            return;
        }

        world.despawn(self.pheromone_entity);

        world
            .resource_mut::<PheromoneMap<Z>>()
            .map
            .remove(&self.position);
    }
}
