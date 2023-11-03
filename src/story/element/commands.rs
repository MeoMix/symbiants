use super::{AirElementBundle, DirtElementBundle, Element, FoodElementBundle, SandElementBundle};
use crate::story::{
    common::{position::Position, IdMap},
    // TODO: element shouldn't couple to gravity, want to be able to reuse element
    nest_simulation::{gravity::Unstable, nest::Nest},
    grid::Grid
};
use bevy::{ecs::system::Command, prelude::*};

pub trait ElementCommandsExt {
    fn replace_element(&mut self, position: Position, element: Element, target_element: Entity);
    fn spawn_element(&mut self, position: Position, element: Element);
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
        let existing_entity = match world
            .query::<&Grid>()
            .single(world)
            .elements()
            .get_element_entity(self.position)
        {
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
            .query::<&mut Grid>()
            .single_mut(world)
            .elements_mut()
            .set_element(self.position, entity);
    }
}

struct SpawnElementCommand {
    element: Element,
    position: Position,
}

impl Command for SpawnElementCommand {
    fn apply(self, world: &mut World) {
        if let Some(existing_entity) = world
            .query::<&Grid>()
            .single(world)
            .elements()
            .get_element_entity(self.position)
        {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = spawn_element(self.element, self.position, world);
        world
            .query::<&mut Grid>()
            .single_mut(world)
            .elements_mut()
            .set_element(self.position, entity);
    }
}

pub fn spawn_element(element: Element, position: Position, world: &mut World) -> Entity {
    let (id, entity) = match element {
        Element::Air => {
            let element_bundle = AirElementBundle::new(position);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
        Element::Dirt => {
            // HACK: Dirt that spawns below surface level is not unstable but dirt that is above is unstable.
            // It should be possible to do this is a more generic way, but performance issues abound. The main one is
            // is that using a Query which iterates over Element and filters on With<Added> still iterates all elements.
            let nest = world.query::<&Nest>().single(world);
            let element_bundle = DirtElementBundle::new(position);
            let element_bundle_id = element_bundle.id.clone();

            let entity;
            if nest.is_underground(&position) {
                entity = world.spawn(element_bundle).id();
            } else {
                entity = world.spawn((element_bundle, Unstable)).id();
            }

            (element_bundle_id, entity)
        }
        Element::Sand => {
            let element_bundle = SandElementBundle::new(position);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
        Element::Food => {
            let element_bundle = FoodElementBundle::new(position);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
    };

    world.resource_mut::<IdMap>().0.insert(id.clone(), entity);

    entity
}

struct ToggleElementCommand<C: Component + Send + Sync + 'static> {
    position: Position,
    target_element_entity: Entity,
    toggle: bool,
    component: C,
}

impl<C: Component + Send + Sync + 'static> Command for ToggleElementCommand<C> {
    fn apply(self, world: &mut World) {
        let nest = world.query::<&Grid>().single(world);
        let element_entity = match nest.elements().get_element_entity(self.position) {
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
