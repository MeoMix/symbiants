use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub mod ui;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

// TODO: Move this since I renamed it from Label
#[derive(Component)]
pub struct AntLabel(pub Entity);

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
    query.iter().find(|(_, id)| **id == target_id).map(|(entity, _)| entity)
}