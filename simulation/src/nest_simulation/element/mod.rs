pub mod commands;

use crate::common::{position::Position, Zone};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Element entities are represented by their Element enum, but the value of this enum isn't Queryable.
/// As such, map the Element enum into its values, represented as specific Components, and apply those.
/// This allows systems to Query for Elements of a specific type efficiently.
pub fn map_element_to_marker(
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
