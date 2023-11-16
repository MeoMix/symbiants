use std::marker::PhantomData;

use super::{Element, ElementBundle};
use crate::story::{
    common::{position::Position, IdMap, Zone},
    grid::Grid,
};
use bevy::{ecs::system::Command, prelude::*};

pub trait ElementCommandsExt {
    fn replace_element<Z: Zone>(
        &mut self,
        position: Position,
        element: Element,
        target_element: Entity,
        zone: Z,
    );
    fn spawn_element<Z: Zone>(
        &mut self,
        position: Position,
        element: Element,
        zone: Z,
    );
    fn toggle_element_command<C: Component, Z: Zone>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
        zone: PhantomData<Z>,
    );
}

impl<'w, 's> ElementCommandsExt for Commands<'w, 's> {
    fn replace_element<Z: Zone>(
        &mut self,
        position: Position,
        element: Element,
        target_element: Entity,
        zone: Z,
    ) {
        self.add(ReplaceElementCommand {
            position,
            target_element,
            element,
            zone,
        })
    }

    fn spawn_element<Z: Zone>(
        &mut self,
        position: Position,
        element: Element,
        zone: Z,
    ) {
        self.add(SpawnElementCommand {
            element,
            position,
            zone,
        })
    }

    fn toggle_element_command<C: Component, Z: Zone>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
        zone: PhantomData<Z>,
    ) {
        self.add(ToggleElementCommand::<C, Z> {
            target_element_entity,
            position,
            toggle,
            component,
            zone,
        })
    }
}

struct ReplaceElementCommand<Z: Zone> {
    target_element: Entity,
    element: Element,
    position: Position,
    // TODO: maybe just infer this from target_element
    zone: Z,
}

impl<Z: Zone> Command for ReplaceElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let grid = world.query_filtered::<&mut Grid, With<Z>>().single(world);

        let existing_entity = match grid.elements().get_element_entity(self.position) {
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

        let entity = spawn_element(self.element, self.position, self.zone, world);

        let mut grid = world
            .query_filtered::<&mut Grid, With<Z>>()
            .single_mut(world);

        grid.elements_mut().set_element(self.position, entity);
    }
}

struct SpawnElementCommand<Z: Zone> {
    element: Element,
    position: Position,
    zone: Z,
}

impl<Z: Zone> Command for SpawnElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let grid = world.query_filtered::<&mut Grid, With<Z>>().single(world);

        if let Some(existing_entity) = grid.elements().get_element_entity(self.position) {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = spawn_element(self.element, self.position, self.zone, world);

        let mut grid = world
            .query_filtered::<&mut Grid, With<Z>>()
            .single_mut(world);

        grid.elements_mut().set_element(self.position, entity);
    }
}

pub fn spawn_element<Z: Zone>(
    element: Element,
    position: Position,
    zone: Z,
    world: &mut World,
) -> Entity {
    let element_bundle = ElementBundle::new(element, position, zone);
    let id = element_bundle.id.clone();
    let entity = world.spawn(element_bundle).id();

    world.resource_mut::<IdMap>().0.insert(id, entity);

    entity
}

struct ToggleElementCommand<C: Component, Z: Zone> {
    position: Position,
    target_element_entity: Entity,
    toggle: bool,
    component: C,
    zone: PhantomData<Z>,
}

impl<C: Component, Z: Zone> Command for ToggleElementCommand<C, Z> {
    fn apply(self, world: &mut World) {
        let grid = world.query_filtered::<&mut Grid, With<Z>>().single(world);

        let element_entity = match grid.elements().get_element_entity(self.position) {
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
