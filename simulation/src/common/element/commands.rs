use super::{Element, ElementBundle};
use crate::common::{
    grid::{GridElements, GridElementsMut},
    position::Position,
    Zone,
};
use bevy::{
    ecs::{system::SystemState, world::Command},
    prelude::*,
};
use std::marker::PhantomData;

pub trait ElementCommandsExt {
    fn replace_element<Z: Zone>(
        &mut self,
        position: Position,
        element: Element,
        target_element: Entity,
        zone: Z,
    );
    fn spawn_element<Z: Zone>(&mut self, position: Position, element: Element, zone: Z);
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

    fn spawn_element<Z: Zone>(&mut self, position: Position, element: Element, zone: Z) {
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
    zone: Z,
}

impl<Z: Zone> Command for ReplaceElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<GridElements<Z>> = SystemState::new(world);
        let grid_elements = system_state.get(world);

        let existing_entity = match grid_elements.get_entity(self.position) {
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

        let entity = world
            .spawn(ElementBundle::new(self.element, self.position, self.zone))
            .id();

        let mut system_state: SystemState<GridElementsMut<Z>> = SystemState::new(world);
        let mut grid_elements = system_state.get_mut(world);

        grid_elements.set(self.position, entity);
    }
}

struct SpawnElementCommand<Z: Zone> {
    element: Element,
    position: Position,
    zone: Z,
}

impl<Z: Zone> Command for SpawnElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<GridElements<Z>> = SystemState::new(world);
        let grid_elements = system_state.get(world);

        if let Some(existing_entity) = grid_elements.get_entity(self.position) {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = world
            .spawn(ElementBundle::new(self.element, self.position, self.zone))
            .id();

        let mut system_state: SystemState<GridElementsMut<Z>> = SystemState::new(world);
        let mut grid_elements = system_state.get_mut(world);

        grid_elements.set(self.position, entity);
    }
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
        let mut system_state: SystemState<GridElements<Z>> = SystemState::new(world);
        let grid_elements = system_state.get(world);

        let element_entity = match grid_elements.get_entity(self.position) {
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
