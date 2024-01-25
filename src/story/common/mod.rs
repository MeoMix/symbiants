use bevy::prelude::*;

use self::position::Position;

pub mod position;

// This maps to AtNest or AtCrater
/// Use an empty trait to mark Nest and Crater zones to ensure strong type safety in generic systems.
pub trait Zone: Component {}

pub fn register_common(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Entity>();
    app_type_registry.write().register::<Option<Entity>>();
    app_type_registry.write().register::<Position>();
}

pub fn despawn_model<Model: Component>(
    model_query: Query<Entity, With<Model>>,
    mut commands: Commands,
) {
    for model_entity in model_query.iter() {
        commands.entity(model_entity).despawn();
    }
}
