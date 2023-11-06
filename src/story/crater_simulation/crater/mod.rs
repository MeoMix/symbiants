pub mod ui;

use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

use crate::{
    settings::Settings,
    story::{
        common::{position::Position, register, Id},
        element::Element,
        grid::{elements_cache::ElementsCache, Grid},
    },
};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtCrater;

/// Note the intentional omission of reflection/serialization.
/// This is because Crater is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Crater;

pub fn register_crater(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Crater>(&app_type_registry, &mut saveable_registry);
    register::<AtCrater>(&app_type_registry, &mut saveable_registry);
}

pub fn setup_crater(mut commands: Commands) {
    commands.spawn((Crater, Id::default()));
}

pub fn setup_crater_grid(
    element_query: Query<(&mut Position, Entity), With<Element>>,
    crater_query: Query<Entity, With<Crater>>,
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

    commands.entity(crater_query.single()).insert((Grid::new(
        settings.crater_width,
        settings.crater_height,
        ElementsCache::new(elements_cache),
    ),));
}

pub fn teardown_crater(mut commands: Commands, crater_entity_query: Query<Entity, With<Crater>>) {
    let crater_entity = crater_entity_query.single();

    commands.entity(crater_entity).despawn();
}
