use core::panic;

use bevy::{ecs::system::Command, prelude::*};

use crate::{
    ant::AntInventory,
    common::{Id, IdMap},
    element::{commands::spawn_element, AirElementBundle, Element},
    nest::{position::Position, Nest}, settings::Settings,
};

use super::{
    nesting::Nesting, AntBundle, AntColor, AntName, AntOrientation, AntRole, Initiative,
    InventoryItemBundle, hunger::Hunger, Ant, digestion::Digestion,
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
    );
    fn dig(&mut self, ant_entity: Entity, target_position: Position, target_element_entity: Entity);
    fn drop(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
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
    ) {
        self.add(SpawnAntCommand {
            position,
            color,
            orientation,
            inventory,
            role,
            name,
            initiative,
        });
    }

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
        let nest = world.resource::<Nest>();
        let element_entity = match nest.get_element_entity(self.target_position) {
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
            .resource_mut::<Nest>()
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

        world.resource_mut::<IdMap>().0.insert(inventory_item_element_id.clone(), inventory_item_entity);

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
}

impl Command for DropElementCommand {
    fn apply(self, world: &mut World) {
        let nest = world.resource::<Nest>();
        let air_entity = match nest.get_element_entity(self.target_position) {
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
            .resource_mut::<Nest>()
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
}

impl Command for SpawnAntCommand {
    fn apply(self, world: &mut World) {
        let settings = world.resource::<Settings>();

        let ant_bundle = AntBundle {
            id: Id::default(),
            ant: Ant,
            position: self.position,
            orientation: self.orientation,
            inventory: self.inventory,
            role: self.role,
            initiative: self.initiative,
            name: self.name,
            color: self.color,
            hunger: Hunger::new(settings.max_hunger_time),
            digestion: Digestion::new(settings.max_digestion_time),
        };

        let id = ant_bundle.id.clone();

        let entity;
        if self.role == AntRole::Queen {
            // TODO: It's weird to have one special default property for Queen?
            entity = world.spawn((ant_bundle, Nesting::default())).id();
        } else {
            entity = world.spawn(ant_bundle).id();
        }
        
        world
            .resource_mut::<IdMap>()
            .0
            .insert(id, entity);
    }
}
