use core::panic;

use bevy::{prelude::*, ecs::system::Command};

use crate::{map::{Position, WorldMap}, element::{Element, AirElementBundle, commands::spawn_element}, ant::AntInventory};

pub trait AntCommandsExt {
    fn dig_element(&mut self, ant: Entity, position: Position, target_element: Entity);
    fn drop_element(&mut self, ant: Entity, position: Position, target_element: Entity);
}

impl<'w, 's> AntCommandsExt for Commands<'w, 's> {
    fn dig_element(&mut self, ant: Entity, position: Position, target_element: Entity) {
        self.add(DigElementCommand {
            ant,
            position,
            target_element,
        })
    }

    fn drop_element(&mut self, ant: Entity, position: Position, target_element: Entity) {
        self.add(DropElementCommand {
            ant,
            position,
            target_element,
        })
    }
}

struct DigElementCommand {
    // TODO: I'm not sure if it would be nice/good, but having this depend on AntInventory rather than Entity might express its public dependencies more clearly?
    // The entity issuing the command.
    ant: Entity,
    // The entity expected to be affected by the command.
    target_element: Entity,
    // The location of the target element.
    position: Position,
}

impl Command for DigElementCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let element_entity = match world_map.get_element(self.position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if element_entity != self.target_element {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        let element = match world.get::<Element>(element_entity) {
            Some(element) => *element,
            None => {
                info!("Failed to get Element component for element entitity {:?}.", element_entity);
                return;
            },
        };
        
        world.entity_mut(element_entity).despawn();
            
        let air_entity = world.spawn(AirElementBundle::new(self.position)).id();
        world.resource_mut::<WorldMap>().set_element(self.position, air_entity);

        // TODO: There's probably a more elegant way to express this - "denseness" of sand rather than changing between dirt/sand.
        let mut inventory_element = element;
        if inventory_element == Element::Dirt {
            inventory_element = Element::Sand;
        }

        match world.get_mut::<AntInventory>(self.ant) {
            Some(mut inventory) => {
                *inventory = AntInventory(Some(inventory_element));
                info!("Ant {:?} inventory: {:?}", self.ant, inventory);
            },
            None => panic!("Failed to get inventory for ant {:?}", self.ant),
        };
    }
}

struct DropElementCommand {
    // The entity issuing the command.
    ant: Entity,
    // The entity expected to be affected by the command.
    target_element: Entity,
    // The location of the target element.
    position: Position,
}

impl Command for DropElementCommand {
    fn apply(self, world: &mut World) {
        let world_map = world.resource::<WorldMap>();
        let air_entity = match world_map.get_element(self.position) {
            Some(entity) => *entity,
            None => {
                info!("No entity found at position {:?}", self.position);
                return;
            }
        };

        if air_entity != self.target_element {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        world.entity_mut(air_entity).despawn();

        let inventory = match world.get::<AntInventory>(self.ant) {
            Some(inventory) => inventory,
            None => panic!("Failed to get inventory for ant {:?}", self.ant),
        };

        let element_entity = match inventory.0 {
            Some(element) => spawn_element(element, self.position, world),
            None => panic!("Ant {:?} has no element in inventory", self.ant),
        };

        world.resource_mut::<WorldMap>().set_element(self.position, element_entity);

        match world.get_mut::<AntInventory>(self.ant) {
            Some(mut inventory) => *inventory = AntInventory(None),
            None => panic!("Failed to get inventory for ant {:?}", self.ant),
        };
    }
}

// TODO: command for eating food