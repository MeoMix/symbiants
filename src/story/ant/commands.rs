use core::panic;

use bevy::{ecs::system::Command, prelude::*};

use crate::story::{
    ant::AntInventory,
    common::{position::Position, Id, IdMap, Location},
    crater_simulation::crater::Crater,
    element::{commands::spawn_element, ElementBundle, Element},
    grid::Grid,
    nest_simulation::nest::Nest,
};

use crate::settings::Settings;

use super::{
    digestion::Digestion, hunger::Hunger, nesting::Nesting, Ant, AntColor, AntName, AntOrientation,
    AntRole, Initiative, InventoryItemBundle,
};

pub trait AntCommandsExt {
    fn spawn_ant(
        &mut self,
        position: Position,
        color: AntColor,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        location: Location,
    );
    fn dig(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        location: Location,
    );
    fn drop(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        location: Location,
    );
}

impl<'w, 's> AntCommandsExt for Commands<'w, 's> {
    fn spawn_ant(
        &mut self,
        position: Position,
        color: AntColor,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        location: Location,
    ) {
        self.add(SpawnAntCommand {
            position,
            color,
            orientation,
            inventory,
            role,
            name,
            initiative,
            location,
        });
    }

    fn dig(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        location: Location,
    ) {
        self.add(DigElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
            location,
        });
    }

    fn drop(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        location: Location,
    ) {
        self.add(DropElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
            location,
        });
    }
}

struct DigElementCommand {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
    location: Location,
}

// TODO: Confirm that ant and element are adjacent to one another at time action is taken.
impl Command for DigElementCommand {
    fn apply(self, world: &mut World) {
        let grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single(world),
        };

        let element_entity = match grid.elements().get_element_entity(self.target_position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.target_position);
                return;
            }
        };

        if element_entity != self.target_element_entity {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        let element = match world.get::<Element>(element_entity) {
            Some(element) => *element,
            None => {
                info!(
                    "Failed to get Element component for element entitity {:?}.",
                    element_entity
                );
                return;
            }
        };

        world.entity_mut(element_entity).despawn();

        let air_entity = world
            .spawn(ElementBundle::new(Element::Air, self.target_position, Location::Nest))
            .id();

        let mut grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single_mut(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single_mut(world),
        };

        grid.elements_mut()
            .set_element(self.target_position, air_entity);

        // TODO: There's probably a more elegant way to express this - "denseness" of sand rather than changing between dirt/sand.
        let mut inventory_element = element;
        if inventory_element == Element::Dirt {
            inventory_element = Element::Sand;
        }

        let mut id_query = world.query::<(Entity, &Id)>();
        let ant_id = id_query
            .iter(world)
            .find(|(entity, _)| *entity == self.ant_entity)
            .map(|(_, id)| id)
            .unwrap();

        let inventory_item_bundle = InventoryItemBundle::new(inventory_element, ant_id.clone());
        let inventory_item_element_id = inventory_item_bundle.id.clone();

        let inventory_item_entity = world.spawn(inventory_item_bundle).id();

        world
            .resource_mut::<IdMap>()
            .0
            .insert(inventory_item_element_id.clone(), inventory_item_entity);

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => inventory.0 = Some(inventory_item_element_id),
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };

        match world.get_mut::<Initiative>(self.ant_entity) {
            Some(mut initiative) => initiative.consume(),
            None => panic!("Failed to get initiative for ant {:?}", self.ant_entity),
        };
    }
}

struct DropElementCommand {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
    location: Location,
}

impl Command for DropElementCommand {
    fn apply(self, world: &mut World) {
        let grid = match self.location {
            Location::Nest => world
                .query_filtered::<&mut Grid, With<Nest>>()
                .single(world),
            Location::Crater => world
                .query_filtered::<&mut Grid, With<Crater>>()
                .single(world),
        };

        let air_entity = match grid.elements().get_element_entity(self.target_position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.target_position);
                return;
            }
        };

        if air_entity != self.target_element_entity {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        // Remove air element from world.
        world.entity_mut(air_entity).despawn();

        let inventory = match world.get::<AntInventory>(self.ant_entity) {
            Some(inventory) => inventory,
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };

        let inventory_item_id = match inventory.0.clone() {
            Some(element_id) => element_id,
            None => panic!("Ant {:?} has no element in inventory", self.ant_entity),
        };

        let mut id_query = world.query::<(Entity, &Id)>();
        let inventory_item_entity = id_query
            .iter(world)
            .find(|(_, id)| **id == inventory_item_id)
            .map(|(entity, _)| entity)
            .unwrap();

        let element = world.get::<Element>(inventory_item_entity).unwrap();

        // Add element to world.
        let element_entity = spawn_element(*element, self.target_position, Location::Nest, world);

        world
            .query_filtered::<&mut Grid, With<Nest>>()
            .single_mut(world)
            .elements_mut()
            .set_element(self.target_position, element_entity);

        // Remove element from ant inventory.
        world.entity_mut(inventory_item_entity).despawn();

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => inventory.0 = None,
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };

        match world.get_mut::<Initiative>(self.ant_entity) {
            Some(mut initiative) => initiative.consume(),
            // This isn't an exceptional scenario because ants which die lose initative entirely and must drop their inventory.
            // Conceptually, placing an item takes initative but dropping from lack of ability does not.
            None => info!("Failed to get initiative for ant {:?}", self.ant_entity),
        };
    }
}

struct SpawnAntCommand {
    position: Position,
    color: AntColor,
    orientation: AntOrientation,
    inventory: AntInventory,
    role: AntRole,
    name: AntName,
    initiative: Initiative,
    location: Location,
}

impl Command for SpawnAntCommand {
    fn apply(self, world: &mut World) {
        let settings = world.resource::<Settings>();
        let id = Id::default();

        let entity = world
            .spawn((
                id.clone(),
                Ant,
                self.position,
                self.orientation,
                self.inventory,
                self.role,
                self.initiative,
                self.name,
                self.color,
                self.location,
                Hunger::new(settings.max_hunger_time),
                Digestion::new(settings.max_digestion_time),
            ))
            .id();

        if self.role == AntRole::Queen {
            world.entity_mut(entity).insert(Nesting::default());
        }

        world.resource_mut::<IdMap>().0.insert(id, entity);
    }
}
