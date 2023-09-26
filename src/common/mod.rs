use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::grid::position::Position;

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
pub fn register<T: GetTypeRegistration>(
    app_type_registry: &ResMut<AppTypeRegistry>,
    saveable_registry: &mut ResMut<SaveableRegistry>,
) {
    // Enable reflection
    app_type_registry.write().register::<T>();

    // Enable serialization
    saveable_registry.register::<T>();
}

pub fn initialize_common(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Id>(&app_type_registry, &mut saveable_registry);
    register::<Option<Id>>(&app_type_registry, &mut saveable_registry);
    register::<Uuid>(&app_type_registry, &mut saveable_registry);
    register::<Option<Position>>(&app_type_registry, &mut saveable_registry);
    register::<Position>(&app_type_registry, &mut saveable_registry);
}

pub fn deinitialize_common() {}
