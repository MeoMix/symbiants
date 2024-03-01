pub mod commands;
pub mod death;
pub mod digestion;
pub mod hunger;
pub mod initiative;
// pub mod sleep;
mod name_list;

use self::{digestion::Digestion, hunger::Hunger, initiative::Initiative, name_list::get_random_name};
use crate::common::{element::Element, position::Position, Zone};
use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

#[derive(Bundle)]
pub struct AntBundle<Z>
where
    Z: Zone,
{
    ant: Ant,
    position: Position,
    role: AntRole,
    initiative: Initiative,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
    digestion: Digestion,
    inventory: AntInventory,
    zone: Z,
}

impl<Z: Zone> AntBundle<Z> {
    pub fn new(
        position: Position,
        color: AntColor,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
        hunger: Hunger,
        digestion: Digestion,
    ) -> Self {
        Self {
            ant: Ant,
            position,
            color,
            inventory,
            role,
            name,
            initiative,
            zone,
            hunger,
            digestion,
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntName(pub String);

impl AntName {
    pub fn random(rng: &mut Mut<GlobalRng>) -> Self {
        AntName(get_random_name(&mut rng.reborrow()))
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntColor(pub Color);

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct AntInventory(pub Option<Entity>);

impl MapEntities for AntInventory {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        if let Some(entity) = self.0 {
            self.0 = Some(entity_mapper.get_or_reserve(entity));
        }
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Dead;

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Ant;

impl Ant {}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum AntRole {
    #[default]
    Worker,
    Queen,
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryItem;

#[derive(Bundle)]
pub struct InventoryItemBundle<Z>
where
    Z: Zone,
{
    element: Element,
    inventory_item: InventoryItem,
    zone: Z,
}

impl<Z: Zone> InventoryItemBundle<Z> {
    pub fn new(element: Element, zone: Z) -> Self {
        InventoryItemBundle {
            element,
            inventory_item: InventoryItem,
            zone,
        }
    }
}

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Ant>();
    app_type_registry.write().register::<AntName>();
    app_type_registry.write().register::<AntColor>();
    app_type_registry.write().register::<Initiative>();
    app_type_registry.write().register::<AntRole>();
    app_type_registry.write().register::<AntInventory>();
    app_type_registry.write().register::<InventoryItem>();

    app_type_registry.write().register::<Dead>();
    app_type_registry.write().register::<Hunger>();
    app_type_registry.write().register::<Digestion>();

    // TODO: This might be nest-specific, but maybe needs to be supported at crater just in case
    // app_type_registry.write().register::<Asleep>();
}