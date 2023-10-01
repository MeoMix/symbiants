use core::panic;

use bevy::{ecs::system::Command, prelude::*};

use crate::{
    ant::AntInventory,
    common::Id,
    element::{commands::spawn_element, AirElementBundle, Element},
    world_map::{position::Position, WorldMap},
};

use super::InventoryItemBundle;

pub trait AntCommandsExt {
    fn dig(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity);
    fn drop(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
    );
}

impl<'w, 's> AntCommandsExt for Commands<'w, 's> {
    fn dig(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
    ) {
        self.add(DigElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
        });
    }

    fn drop(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
    ) {
        self.add(DropElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
        });
    }
}

struct DigElementCommand {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
}

// TODO: Confirm that ant and element are adjacent to one another at time action is taken.
impl Command for DigElementCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let element_entity = match world_map.get_element(self.target_position) {
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
            .spawn(AirElementBundle::new(self.target_position))
            .id();
        world
            .resource_mut::<WorldMap>()
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

        world.spawn(inventory_item_bundle);

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => {
                inventory.0 = Some(inventory_item_element_id);
            }
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };
    }
}

struct DropElementCommand {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
}

impl Command for DropElementCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let air_entity = match world_map.get_element(self.target_position) {
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
        let element_entity = spawn_element(*element, self.target_position, world);

        world
            .resource_mut::<WorldMap>()
            .set_element(self.target_position, element_entity);

        // Remove element from ant inventory.
        world.entity_mut(inventory_item_entity).despawn();

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => inventory.0 = None,
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };
    }
}
