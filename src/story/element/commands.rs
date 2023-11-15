use super::{AirElementBundle, DirtElementBundle, Element, FoodElementBundle, SandElementBundle};
use crate::story::{
    common::{position::Position, IdMap, Location},
    crater_simulation::crater::Crater,
    grid::Grid,
    nest_simulation::nest::Nest,
};
use bevy::{ecs::system::Command, prelude::*};

pub trait ElementCommandsExt {
    fn replace_element(
        &mut self,
        position: Position,
        element: Element,
        target_element: Entity,
        location: Location,
    );
    fn spawn_element(&mut self, position: Position, element: Element, location: Location);
    fn toggle_element_command<C: Component + Send + Sync + 'static>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
        location: Location,
    );
}

impl<'w, 's> ElementCommandsExt for Commands<'w, 's> {
    fn replace_element(
        &mut self,
        position: Position,
        element: Element,
        target_element: Entity,
        location: Location,
    ) {
        self.add(ReplaceElementCommand {
            position,
            target_element,
            element,
            location,
        })
    }

    fn spawn_element(&mut self, position: Position, element: Element, location: Location) {
        self.add(SpawnElementCommand {
            element,
            position,
            location,
        })
    }

    fn toggle_element_command<C: Component + Send + Sync + 'static>(
        &mut self,
        target_element_entity: Entity,
        position: Position,
        toggle: bool,
        component: C,
        location: Location,
    ) {
        self.add(ToggleElementCommand::<C> {
            target_element_entity,
            position,
            toggle,
            component,
            location,
        })
    }
}

struct ReplaceElementCommand {
    target_element: Entity,
    element: Element,
    position: Position,
    // TODO: maybe just infer this from target_element
    location: Location,
}

impl Command for ReplaceElementCommand {
    fn apply(self, world: &mut World) {
        let grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single(world),
        };

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

        let entity = spawn_element(self.element, self.position, self.location, world);

        let mut grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single_mut(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single_mut(world),
        };

        grid.elements_mut().set_element(self.position, entity);
    }
}

struct SpawnElementCommand {
    element: Element,
    position: Position,
    location: Location,
}

impl Command for SpawnElementCommand {
    fn apply(self, world: &mut World) {
        let grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single(world),
        };

        if let Some(existing_entity) = grid.elements().get_element_entity(self.position) {
            info!(
                "Entity {:?} already exists at position {:?}",
                existing_entity, self.position
            );
            return;
        }

        let entity = spawn_element(self.element, self.position, self.location, world);

        let mut grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single_mut(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single_mut(world),
        };

        grid.elements_mut().set_element(self.position, entity);
    }
}

pub fn spawn_element(
    element: Element,
    position: Position,
    location: Location,
    world: &mut World,
) -> Entity {
    let (id, entity) = match element {
        Element::Air => {
            let element_bundle = AirElementBundle::new(position, location);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
        Element::Dirt => {
            let element_bundle = DirtElementBundle::new(position, location);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
        Element::Sand => {
            let element_bundle = SandElementBundle::new(position, location);
            let element_bundle_id = element_bundle.id.clone();

            let entity = world.spawn(element_bundle).id();

            (element_bundle_id, entity)
        }
        Element::Food => {
            let element_bundle = FoodElementBundle::new(position, location);
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
    location: Location,
}

impl<C: Component + Send + Sync + 'static> Command for ToggleElementCommand<C> {
    fn apply(self, world: &mut World) {
        let grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single(world),
        };

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
