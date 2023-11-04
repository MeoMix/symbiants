use bevy::prelude::*;

use crate::{
    settings::Settings,
    story::{
        common::position::Position,
        element::Element,
        grid::{elements_cache::ElementsCache, Grid},
    },
};

/// Note the intentional omission of reflection/serialization.
/// This is because Crater is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug)]
pub struct Crater;

pub fn setup_crater(
    element_query: Query<(&mut Position, Entity), With<Element>>,
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

    commands.spawn((
        // VisibleGrid,
        // TODO: prefer stronger UI separation, can't reactively add this without getting spammed with warnings when setting up children.
        SpatialBundle {
            visibility: Visibility::Hidden,
            ..default()
        },
        Grid::new(
            settings.crater_width,
            settings.crater_height,
            ElementsCache::new(elements_cache),
        ),
        Crater,
    ));
}

pub fn teardown_crater(mut commands: Commands, crater_entity_query: Query<Entity, With<Crater>>) {
    let crater_entity = crater_entity_query.single();

    commands.entity(crater_entity).despawn();
}
