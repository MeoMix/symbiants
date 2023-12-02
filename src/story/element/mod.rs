use crate::story::common::position::Position;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::common::Zone;

pub mod commands;
pub mod ui;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Air;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Dirt;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Sand;
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Food;

#[derive(
    Component, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub enum Element {
    #[default]
    Air,
    Dirt,
    Sand,
    Food,
}

impl Element {
    pub fn is_diggable(&self) -> bool {
        match self {
            Element::Dirt => true,
            Element::Sand => true,
            Element::Food => true,
            Element::Air => false,
        }
    }
}

#[derive(Bundle)]
pub struct ElementBundle<Z>
where
    Z: Zone,
{
    element: Element,
    position: Position,
    zone: Z,
}

impl<Z: Zone> ElementBundle<Z> {
    pub fn new(element: Element, position: Position, zone: Z) -> Self {
        Self {
            element,
            position,
            zone,
        }
    }
}

pub fn register_element(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Element>();
    app_type_registry.write().register::<Air>();
    app_type_registry.write().register::<Food>();
    app_type_registry.write().register::<Dirt>();
    app_type_registry.write().register::<Sand>();
}

// TODO: filter?
pub fn teardown_element(mut commands: Commands, element_query: Query<Entity, With<Element>>) {
    for element_entity in element_query.iter() {
        commands.entity(element_entity).despawn_recursive();
    }
}

/// Element entities are represented by their Element enum, but the value of this enum isn't Queryable.
/// As such, denormalize the Element enum into its values, represented as specific Components, and apply those.
/// This allows systems to Query for Elements of a specific type efficiently.
pub fn denormalize_element(
    element_query: Query<
        (Entity, &Element),
        (Without<Air>, Without<Dirt>, Without<Sand>, Without<Food>),
    >,
    mut commands: Commands,
) {
    for (entity, element) in element_query.iter() {
        match element {
            Element::Air => {
                commands.entity(entity).insert(Air);
            }
            Element::Dirt => {
                commands.entity(entity).insert(Dirt);
            }
            Element::Sand => {
                commands.entity(entity).insert(Sand);
            }
            Element::Food => {
                commands.entity(entity).insert(Food);
            }
        }
    }
}
