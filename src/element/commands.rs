use bevy::{ecs::system::Command, prelude::*};
use crate::{map::{Position, WorldMap}, gravity::Unstable};
use super::{Element, FoodElementBundle, SandElementBundle, DirtElementBundle, AirElementBundle};

pub trait ElementCommandsExt {
    fn replace_element(&mut self, position: Position, target_element: Entity, element: Element);
    fn spawn_element(&mut self, element: Element, position: Position);
    fn toggle_element_unstable(&mut self, target_element_entity: Entity, position: Position, toggle: bool);
}

impl<'w, 's> ElementCommandsExt for Commands<'w, 's> {
    fn replace_element(&mut self, position: Position, target_element: Entity, element: Element) {
        self.add(ReplaceElementCommand {
            position,
            target_element,
            element,
        })
    }

    fn spawn_element(&mut self, element: Element, position: Position) {
        self.add(SpawnElementCommand { element, position })
    }

    
    fn toggle_element_unstable(&mut self, target_element_entity: Entity, position: Position, toggle: bool) {
        self.add(MarkElementUnstableCommand { target_element_entity, position, toggle })
    }
}

struct ReplaceElementCommand {
    target_element: Entity,
    element: Element,
    position: Position,
}

impl Command for ReplaceElementCommand {
    fn apply(self, world: &mut World) {
        // The act of replacing an element with another element is delayed because spawn/despawn are queued actions which
        // apply after a system finishes running. It's possible for two writes to occur to the same location during a given
        // system run and, in this scenario, overwrites should not occur because validity checks have already been performed.
        // So, we anticipate the entity to be destroyed and confirm it still exists in the position expected. Otherwise, no-op.
        let world_map = world.resource::<WorldMap>();

        let existing_entity = match world_map.get_element(self.position) {
            Some(entity) => entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if *existing_entity != self.target_element {
            info!("Existing entity doesn't match the current entity.");
            return;
        }

        info!("Despawning entity {:?} at position {:?}", existing_entity, self.position);

        world.entity_mut(*existing_entity).despawn();

        let entity = spawn_element(self.element, self.position, world);
        let mut world_map = world.resource_mut::<WorldMap>();

        world_map.set_element(self.position, entity);
    }
}

struct SpawnElementCommand {
    element: Element,
    position: Position,
}

impl Command for SpawnElementCommand {
    fn apply(self, world: &mut World) {
        if let Some(existing_entity) = world.resource::<WorldMap>().get_element(self.position) {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = spawn_element(self.element, self.position, world);
        world
            .resource_mut::<WorldMap>()
            .set_element(self.position, entity);
    }
}

pub fn spawn_element(element: Element, position: Position, world: &mut World) -> Entity {
    match element {
        Element::Air => world.spawn(AirElementBundle::new(position)).id(),
        Element::Dirt => world.spawn(DirtElementBundle::new(position)).id(),
        Element::Sand => world.spawn(SandElementBundle::new(position)).id(),
        Element::Food => world.spawn(FoodElementBundle::new(position)).id(),
    }
}

// TODO: Make this more generic so singular operations against elements can be generally safeguarded?
struct MarkElementUnstableCommand {
    target_element_entity: Entity,
    position: Position,
    toggle: bool,
}

impl Command for MarkElementUnstableCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let element_entity = match world_map.get_element(self.position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if element_entity != self.target_element_entity {
            info!("Entity at position does not match expected entity.");
            return;
        }

        if self.toggle {
            world.entity_mut(element_entity).insert(Unstable);
        } else {
            world.entity_mut(element_entity).remove::<Unstable>();
        }
    }
}