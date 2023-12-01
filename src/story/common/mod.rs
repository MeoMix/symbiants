use bevy::{prelude::*, reflect::GetTypeRegistration};
use uuid::Uuid;

use self::position::Position;

pub mod position;
pub mod ui;

/// Use an empty trait to mark Nest and Crater zones to ensure strong type safety in generic systems.
pub trait Zone: Component {}

/// Register a given type such that it is valid to use with `bevy_save`.
pub fn register<T: GetTypeRegistration>(app_type_registry: &ResMut<AppTypeRegistry>) {
    // Enable reflection
    app_type_registry.write().register::<T>();
}

pub fn register_common(app_type_registry: ResMut<AppTypeRegistry>) {
    register::<Entity>(&app_type_registry);
    register::<Option<Entity>>(&app_type_registry);
    register::<Uuid>(&app_type_registry);
    register::<Position>(&app_type_registry);
}
