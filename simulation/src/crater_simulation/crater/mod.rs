use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::common::Zone;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtCrater;

impl Zone for AtCrater {}

/// Note the intentional omission of reflection/serialization.
/// This is because Crater is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Crater;

pub fn register_crater(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Crater>();
    app_type_registry.write().register::<AtCrater>();
}

pub fn spawn_crater(mut commands: Commands) {
    commands.spawn((Crater, AtCrater));
}