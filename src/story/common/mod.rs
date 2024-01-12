use bevy::prelude::*;

use self::{
    position::Position,
    ui::{ModelViewEntityMap, SelectedEntity, SelectionSprite},
};

pub mod position;
pub mod ui;

// This maps to AtNest or AtCrater
/// Use an empty trait to mark Nest and Crater zones to ensure strong type safety in generic systems.
pub trait Zone: Component {}

pub fn register_common(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Entity>();
    app_type_registry.write().register::<Option<Entity>>();
    app_type_registry.write().register::<Position>();
}

pub fn setup_common(mut commands: Commands) {
    commands.init_resource::<ModelViewEntityMap>();
    commands.init_resource::<SelectedEntity>();
}

pub fn teardown_common(
    selection_sprite_query: Query<Entity, With<SelectionSprite>>,
    mut commands: Commands,
) {
    if let Ok(selection_sprite_entity) = selection_sprite_query.get_single() {
        commands.entity(selection_sprite_entity).despawn();
    }

    commands.remove_resource::<SelectedEntity>();
}
