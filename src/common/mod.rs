use bevy::{prelude::*, reflect::GetTypeRegistration, utils::HashMap};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::nest::position::Position;

pub mod ui;

// TODO: clean up IdMap on Id component removal.
/// Note the intentional omission of reflection/serialization.
/// This is because IdMap is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Resource, Debug, Default)]
pub struct IdMap(pub HashMap<Id, Entity>);

// Id is needed because Entity isn't fit for user across sessions, i.e. save state, refresh page, load state.
#[derive(Component, Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Id(pub Uuid);

impl Default for Id {
    fn default() -> Self {
        Id(Uuid::new_v4())
    }
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

pub fn register_common(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Id>(&app_type_registry, &mut saveable_registry);
    register::<Option<Id>>(&app_type_registry, &mut saveable_registry);
    register::<Uuid>(&app_type_registry, &mut saveable_registry);
    register::<Position>(&app_type_registry, &mut saveable_registry);
}

pub fn pre_setup_common(mut commands: Commands) {
    commands.init_resource::<IdMap>();
}

pub fn setup_common(id_query: Query<(&Id, Entity)>, mut id_map: ResMut<IdMap>) {
    for (id, entity) in id_query.iter() {
        id_map.0.insert(id.clone(), entity);
    }
}
