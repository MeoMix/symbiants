use core::panic;

use bevy::{ecs::system::Command, prelude::*};

use crate::story::simulation::{
    common::{grid::Grid, position::Position, Zone},
    nest_simulation::{
        ant::AntInventory,
        element::{Element, ElementBundle},
    },
};

use crate::settings::Settings;

use super::{
    digestion::Digestion, hunger::Hunger, Ant, AntBundle, AntColor, AntName, AntOrientation,
    AntRole, Initiative, InventoryItemBundle,
};

pub trait AntCommandsExt {
    fn spawn_ant<Z: Zone>(
        &mut self,
        position: Position,
        color: AntColor,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
    );
    fn dig<Z: Zone>(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        zone: Z,
    );
    fn drop<Z: Zone>(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        zone: Z,
    );
}

impl<'w, 's> AntCommandsExt for Commands<'w, 's> {
    fn spawn_ant<Z: Zone>(
        &mut self,
        position: Position,
        color: AntColor,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
    ) {
        self.add(SpawnAntCommand {
            position,
            color,
            orientation,
            inventory,
            role,
            name,
            initiative,
            zone,
        });
    }

    fn dig<Z: Zone>(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        zone: Z,
    ) {
        self.add(DigElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
            zone,
        });
    }

    fn drop<Z: Zone>(
        &mut self,
        ant_entity: Entity,
        target_position: Position,
        target_element_entity: Entity,
        zone: Z,
    ) {
        self.add(DropElementCommand {
            ant_entity,
            target_position,
            target_element_entity,
            zone,
        });
    }
}

struct DigElementCommand<Z: Zone> {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
    zone: Z,
}

// TODO: Confirm that ant and element are adjacent to one another at time action is taken.
impl<Z: Zone> Command for DigElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let grid = world.query_filtered::<&mut Grid, With<Z>>().single(world);

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
            .spawn(ElementBundle::new(
                Element::Air,
                self.target_position,
                self.zone,
            ))
            .id();

        let mut grid = world
            .query_filtered::<&mut Grid, With<Z>>()
            .single_mut(world);

        grid.elements_mut()
            .set_element(self.target_position, air_entity);

        // TODO: There's probably a more elegant way to express this - "denseness" of sand rather than changing between dirt/sand.
        let mut inventory_element = element;
        if inventory_element == Element::Dirt {
            inventory_element = Element::Sand;
        }

        let inventory_item_entity = world
            .spawn(InventoryItemBundle::new(inventory_element))
            .id();

        match world.get_mut::<AntInventory>(self.ant_entity) {
            Some(mut inventory) => inventory.0 = Some(inventory_item_entity),
            None => panic!("Failed to get inventory for ant {:?}", self.ant_entity),
        };

        match world.get_mut::<Initiative>(self.ant_entity) {
            Some(mut initiative) => initiative.consume(),
            None => panic!("Failed to get initiative for ant {:?}", self.ant_entity),
        };
    }
}

struct DropElementCommand<Z: Zone> {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
    zone: Z,
}

impl<Z: Zone> Command for DropElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let grid = world.query_filtered::<&mut Grid, With<Z>>().single(world);

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

        let inventory_item_entity = match inventory.0.clone() {
            Some(element_id) => element_id,
            None => panic!("Ant {:?} has no element in inventory", self.ant_entity),
        };

        let element = world.get::<Element>(inventory_item_entity).unwrap();

        // Add element to world.
        let element_entity = world
            .spawn(ElementBundle::new(
                *element,
                self.target_position,
                self.zone,
            ))
            .id();

        let mut grid = world
            .query_filtered::<&mut Grid, With<Z>>()
            .single_mut(world);

        grid.elements_mut()
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

struct SpawnAntCommand<Z: Zone> {
    position: Position,
    color: AntColor,
    orientation: AntOrientation,
    inventory: AntInventory,
    role: AntRole,
    name: AntName,
    initiative: Initiative,
    zone: Z,
}

// TODO: Get rid of this since I don't need to keep a cache synced anymore
impl<Z: Zone> Command for SpawnAntCommand<Z> {
    fn apply(self, world: &mut World) {
        let settings = world.resource::<Settings>();

        world.spawn(AntBundle {
            ant: Ant,
            position: self.position,
            orientation: self.orientation,
            inventory: self.inventory,
            role: self.role,
            initiative: self.initiative,
            name: self.name,
            color: self.color,
            zone: self.zone,
            hunger: Hunger::new(settings.max_hunger_time),
            digestion: Digestion::new(settings.max_digestion_time),
        });
    }
}
