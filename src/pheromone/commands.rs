use crate::world_map::position::Position;
use bevy::{ecs::system::Command, prelude::*};

use super::{Pheromone, PheromoneMap};

pub trait PheromoneCommandsExt {
    fn spawn_pheromone(&mut self, position: Position, pheromone: Pheromone);
}

impl<'w, 's> PheromoneCommandsExt for Commands<'w, 's> {
    fn spawn_pheromone(&mut self, position: Position, pheromone: Pheromone) {
        self.add(SpawnPheromoneCommand {
            position,
            pheromone,
        })
    }
}

struct SpawnPheromoneCommand {
    position: Position,
    pheromone: Pheromone,
}

impl Command for SpawnPheromoneCommand {
    /// Spawn a new Pheromone entity and update the associate PheromoneMap cache.
    /// Performed in a custom command to provide a transactional wrapper around issuing command + updating cache.
    fn apply(self, world: &mut World) {
        // TODO: maybe overwrite existing pheromone instead of noop?
        if let Some(_) = world.resource::<PheromoneMap>().0.get(&self.position) {
            return;
        }

        let pheromone_entity = world.spawn((self.position, self.pheromone)).id();
        world
            .resource_mut::<PheromoneMap>()
            .0
            .insert(self.position, pheromone_entity);
    }
}
