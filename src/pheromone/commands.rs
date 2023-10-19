use crate::{world_map::position::Position, common::Id};
use bevy::{ecs::system::Command, prelude::*};

use super::{Pheromone, PheromoneMap, PheromoneDuration, PheromoneStrength};

pub trait PheromoneCommandsExt {
    fn spawn_pheromone(&mut self, position: Position, pheromone: Pheromone, pheromone_strength: PheromoneStrength);
    fn despawn_pheromone(&mut self, pheromone_entity: Entity, position: Position);
}

impl<'w, 's> PheromoneCommandsExt for Commands<'w, 's> {
    fn spawn_pheromone(&mut self, position: Position, pheromone: Pheromone, pheromone_strength: PheromoneStrength) {
        self.add(SpawnPheromoneCommand {
            position,
            pheromone,
            pheromone_strength,
        })
    }

    fn despawn_pheromone(&mut self, pheromone_entity: Entity, position: Position) {
        self.add(DespawnPheromoneCommand {
            pheromone_entity,
            position,
        })
    }
}

struct SpawnPheromoneCommand {
    position: Position,
    pheromone: Pheromone,
    pheromone_strength: PheromoneStrength,
}

impl Command for SpawnPheromoneCommand {
    /// Spawn a new Pheromone entity and update the associate PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        // TODO: maybe overwrite existing pheromone instead of noop?
        if let Some(_) = world.resource::<PheromoneMap>().0.get(&self.position) {
            return;
        }

        let pheromone_entity = world
            .spawn((Id::default(), self.position, self.pheromone, PheromoneDuration::default(), self.pheromone_strength))
            .id();
        world
            .resource_mut::<PheromoneMap>()
            .0
            .insert(self.position, pheromone_entity);
    }
}

struct DespawnPheromoneCommand {
    pheromone_entity: Entity,
    position: Position,
}

impl Command for DespawnPheromoneCommand {
    /// Spawn a new Pheromone entity and update the associate PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        if let Some(pheromone_entity) = world.resource::<PheromoneMap>().0.get(&self.position) {
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
            .resource_mut::<PheromoneMap>()
            .0
            .remove(&self.position);
    }
}
