use core::panic;

use bevy::{ecs::system::Command, prelude::*};

use crate::{
    ant::AntInventory,
    element::{commands::spawn_element, AirElementBundle, Element},
    map::{Position, WorldMap},
};

pub trait AntCommandsExt {
    fn dig(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity);
    fn drop(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity);
}

impl<'w, 's> AntCommandsExt for Commands<'w, 's> {
    fn dig(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity) {
        self.add(DigElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
        });
    }

    fn drop(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity) {
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

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => {
                *inventory = AntInventory(Some(inventory_element));
                info!("Ant {:?} inventory: {:?}", self.ant_entity, inventory);
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

        world.entity_mut(air_entity).despawn();

        let inventory = match world.get::<AntInventory>(self.ant_entity) {
            Some(inventory) => inventory,
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };

        let element_entity = match inventory.0 {
            Some(element) => spawn_element(element, self.target_position, world),
            None => panic!("Ant {:?} has no element in inventory", self.ant_entity),
        };

        world
            .resource_mut::<WorldMap>()
            .set_element(self.target_position, element_entity);

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => *inventory = AntInventory(None),
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };
    }
}

// TODO: command for eating food
