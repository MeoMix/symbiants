use super::{AirElementBundle, DirtElementBundle, Element, FoodElementBundle, SandElementBundle};
use crate::map::{Position, WorldMap};
use bevy::{ecs::system::Command, prelude::*};

pub trait ElementCommandsExt {
    fn replace_element(&mut self, position: Position, element: Element, target_element: Entity);
    fn spawn_element(&mut self,  position: Position, element: Element,);
    fn toggle_element_command<C: Component + Send + Sync + 'static>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
    );
}

impl<'w, 's> ElementCommandsExt for Commands<'w, 's> {
    fn replace_element(&mut self, position: Position, element: Element, target_element: Entity) {
        self.add(ReplaceElementCommand {
            position,
            target_element,
            element,
        })
    }

    fn spawn_element(&mut self, position: Position, element: Element) {
        self.add(SpawnElementCommand { element, position })
    }

    fn toggle_element_command<C: Component + Send + Sync + 'static>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
    ) {
        self.add(ToggleElementCommand::<C> {
            target_element_entity,
            position,
            toggle,
            component,
        })
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

        world.entity_mut(*existing_entity).despawn();

        let entity = spawn_element(self.element, self.position, world);
        world
            .resource_mut::<WorldMap>()
            .set_element(self.position, entity);
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

struct ToggleElementCommand<C: Component + Send + Sync + 'static> {
    position: Position,
    target_element_entity: Entity,
    toggle: bool,
    component: C,
}

impl<C: Component + Send + Sync + 'static> Command for ToggleElementCommand<C> {
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
            world.entity_mut(element_entity).insert(self.component);
        } else {
            world.entity_mut(element_entity).remove::<C>();
        }
    }
}
