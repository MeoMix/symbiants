use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        grid::{elements_cache::ElementsCache, Grid},
        position::Position,
        Zone,
    },
    // TODO: Move Element to Common
    nest_simulation::element::Element,
    settings::Settings,
};

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

pub fn insert_crater_grid(
    crater_query: Query<Entity, With<Crater>>,
    element_query: Query<(&mut Position, Entity), (With<Element>, With<AtCrater>)>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let mut elements_cache = vec![
        vec![Entity::PLACEHOLDER; settings.crater_width as usize];
        settings.crater_height as usize
    ];

    for (position, entity) in element_query.iter() {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    commands.entity(crater_query.single()).insert(Grid::new(
        settings.crater_width,
        settings.crater_height,
        ElementsCache::new(elements_cache),
    ));
}
