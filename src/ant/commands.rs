use bevy::{prelude::*, ecs::system::Command};

use crate::{map::{Position, WorldMap}, elements::{Element, AirElementBundle, FoodElementBundle, SandElementBundle}, ant::AntInventory};

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
    // The entity issuing the command.
    ant: Entity,
    // The entity expected to be affected by the command.
    target_element: Entity,
    // The location of the target element.
    position: Position,
}


impl Command for DigElementCommand {
    fn apply(self, world: &mut World) {
        let (element_entity, element) = {
            let world_map = world.resource::<WorldMap>();
            let element_entity = match world_map.get_element(self.position) {
                Some(entity) => *entity,
                None => {
                    info!("No entity found at position {:?}", self.position);
                    return;
                }
            };

            let element = match world.get::<Element>(element_entity) {
                Some(element) => *element,
                None => return,
            };

            (element_entity, element)
        };

        if element_entity != self.target_element {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        // TODO: Does it make sense for this element check to be here?
        if element == Element::Dirt || element == Element::Sand || element == Element::Food {  
            info!("Despawning entity {:?} at position {:?}", element_entity, self.position);
            world.entity_mut(element_entity).despawn();
            
            let air_entity = world.spawn(AirElementBundle::new(self.position)).id();
            let mut world_map = world.resource_mut::<WorldMap>();

            world_map.set_element(self.position, air_entity);

            let mut inventory = match world.get_mut::<AntInventory>(self.ant) {
                Some(inventory) => inventory,
                None => {
                    // TODO: uhh, I can't return here because I already despawned - need full transaction guarantees.
                    info!("Failed to get inventory for ant {:?}", self.ant);
                    return;
                },
            };
            
            if element == Element::Food {
                *inventory = AntInventory(Some(Element::Food));
            } else {
                *inventory = AntInventory(Some(Element::Sand));

                info!("updated inventory to be sand");
            }
        }
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
        let (air_entity, element) = {
            let world_map = world.resource::<WorldMap>();
            let element_entity = match world_map.get_element(self.position) {
                Some(entity) => *entity,
                None => {
                    info!("No entity found at position {:?}", self.position);
                    return;
                }
            };

            let element = match world.get::<Element>(element_entity) {
                Some(element) => *element,
                None => return,
            };

            (element_entity, element)
        };

        if air_entity != self.target_element {
            info!("Existing element entity doesn't match the target element entity.");
            return;
        }

        // TODO: Does it make sense for this element check to be here?
        if element == Element::Air {  
            info!("Despawning air {:?} at position {:?}", air_entity, self.position);
            world.entity_mut(air_entity).despawn();

            let inventory = match world.get::<AntInventory>(self.ant) {
                Some(inventory) => inventory,
                None => {
                    // TODO: Can't return this late after mutations have occurred
                    info!("Failed to get inventory for ant {:?}", self.ant);
                    return;
                },
            };

            let element_entity = match inventory.0 {
                Some(Element::Food) => world.spawn(FoodElementBundle::new(self.position)).id(),
                Some(Element::Sand) => world.spawn(SandElementBundle::new(self.position)).id(),
                _ => panic!("Invalid element {:?}", element),
            };

            let mut world_map = world.resource_mut::<WorldMap>();

            world_map.set_element(self.position, element_entity);

            let mut inventory = match world.get_mut::<AntInventory>(self.ant) {
                Some(inventory) => inventory,
                None => {
                    // TODO: Can't return this late after mutations have occurred
                    info!("Failed to get inventory for ant {:?}", self.ant);
                    return;
                },
            };

            *inventory = AntInventory(None);
        }
    }
}