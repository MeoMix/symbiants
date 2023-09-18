use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::grid::position::Position;

pub mod ui;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Id(pub Uuid);

impl Default for Id {
    fn default() -> Self {
        Id(Uuid::new_v4())
    }
}

// TODO: Use cache instead of iterating all entities
pub fn get_entity_from_id(target_id: Id, query: &Query<(Entity, &Id)>) -> Option<Entity> {
    query
        .iter()
        .find(|(_, id)| **id == target_id)
        .map(|(entity, _)| entity)
}

/// Register a given type such that it is valid to use with `bevy_save`.
pub fn register<T: GetTypeRegistration>(world: &mut World) {
    // Enable reflection
    world
        .resource_mut::<AppTypeRegistry>()
        .write()
        .register::<T>();

    // Enable serialization
    world.resource_mut::<SaveableRegistry>().register::<T>();
}

pub fn initialize_common(world: &mut World) {
    register::<Id>(world);
    register::<Option<Id>>(world);
    register::<Uuid>(world);
    register::<Option<Position>>(world);
    register::<Position>(world);
}

pub fn deinitialize_common() {}
