use crate::{
    common::{
        ant::{
            digestion::Digestion, hunger::Hunger, AntBundle, AntColor, AntInventory, AntName, AntRole, Initiative, InventoryItemBundle
        },
        element::{Element, ElementBundle},
        grid::{GridElements, GridElementsMut},
        position::Position,
        Zone,
    }, crater_simulation::ant::CraterOrientation, nest_simulation::ant::NestOrientation, settings::Settings
};
use bevy::{
    ecs::{system::SystemState, world::Command},
    prelude::*,
};
use core::panic;

pub trait AntCommandsExt {
    fn spawn_ant<Z: Zone>(
        &mut self,
        position: Position,
        color: AntColor,
        // TODO: obviously terrible
        nest_orientation: Option<NestOrientation>,
        crater_orientation: Option<CraterOrientation>,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
    );
    fn dig<Z: Zone + Copy>(
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
        nest_orientation: Option<NestOrientation>,
        crater_orientation: Option<CraterOrientation>,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
    ) {
        self.add(SpawnAntCommand {
            position,
            color,
            nest_orientation,
            crater_orientation,
            inventory,
            role,
            name,
            initiative,
            zone,
        });
    }

    fn dig<Z: Zone + Copy>(
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

struct DigElementCommand<Z: Zone + Copy> {
    ant_entity: Entity,
    target_element_entity: Entity,
    target_position: Position,
    zone: Z,
}

// TODO: Confirm that ant and element are adjacent to one another at time action is taken.
impl<Z: Zone + Copy> Command for DigElementCommand<Z> {
    fn apply(self, world: &mut World) {
        let mut system_state: SystemState<GridElements<Z>> = SystemState::new(world);
        let grid_elements = system_state.get(world);

        let element_entity = match grid_elements.get_entity(self.target_position) {
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

        let mut system_state: SystemState<GridElementsMut<Z>> = SystemState::new(world);
        let mut grid_elements = system_state.get_mut(world);

        grid_elements.set(self.target_position, air_entity);

        // TODO: There's probably a more elegant way to express this - "denseness" of sand rather than changing between dirt/sand.
        let mut inventory_element = element;
        if inventory_element == Element::Dirt {
            inventory_element = Element::Sand;
        }

        let inventory_item_entity = world
            .spawn(InventoryItemBundle::new(inventory_element, self.zone))
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
        let mut system_state: SystemState<GridElements<Z>> = SystemState::new(world);
        let grid_elements = system_state.get(world);

        let air_entity = match grid_elements.get_entity(self.target_position) {
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

        let mut system_state: SystemState<GridElementsMut<Z>> = SystemState::new(world);
        let mut grid_elements = system_state.get_mut(world);

        grid_elements.set(self.target_position, element_entity);

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
    nest_orientation: Option<NestOrientation>,
    crater_orientation: Option<CraterOrientation>,
    inventory: AntInventory,
    role: AntRole,
    name: AntName,
    initiative: Initiative,
    zone: Z,
}


impl<Z: Zone> Command for SpawnAntCommand<Z> {
    fn apply(self, world: &mut World) {
        let settings = world.resource::<Settings>();

        let entity = world.spawn(AntBundle::new(
            self.position,
            self.color,
            // self.nest_orientation,
            // self.crater_orientation,
            self.inventory,
            self.role,
            self.name,
            self.initiative,
            self.zone,
            Hunger::new(settings.max_hunger_time),
            Digestion::new(settings.max_digestion_time),
        )).id();

        if let Some(orientation) = self.nest_orientation {
            world.entity_mut(entity).insert(orientation);
        } else if let Some(orientation) = self.crater_orientation {
            world.entity_mut(entity).insert(orientation);
        } else {
            panic!("Ant must have either a nest or crater orientation.");
        }
    
    }
}
