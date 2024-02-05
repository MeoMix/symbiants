pub mod commands;

use super::nest::AtNest;
use crate::common::{grid::GridElements, position::Position, Zone};
use bevy::{prelude::*, utils::HashSet};
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

// TODO: ***technically*** this is only a view concern but keeping it here for now.
#[derive(Component, Copy, Clone)]
pub struct ElementExposure {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

/// Eagerly calculate which sides of a given Element are exposed to Air.
/// Run against all elements changing position - this supports recalculating on Element removal by responding to Air being added.
pub fn update_element_exposure(
    changed_elements_query: Query<(Entity, &Position, &Element), Changed<Position>>,
    mut commands: Commands,
    grid_elements: GridElements<AtNest>,
) {
    let mut entities = HashSet::new();

    for (entity, position, element) in changed_elements_query.iter() {
        if *element != Element::Air {
            entities.insert((entity, *position));
        }

        for adjacent_position in position.get_adjacent_positions() {
            if let Some(adjacent_element_entity) = grid_elements.get_entity(adjacent_position) {
                let adjacent_element = grid_elements.element(*adjacent_element_entity);

                if *adjacent_element != Element::Air {
                    entities.insert((*adjacent_element_entity, adjacent_position));
                }
            }
        }
    }

    for (entity, position) in entities {
        commands.entity(entity).insert(ElementExposure {
            north: grid_elements.is(position - Position::Y, Element::Air),
            east: grid_elements.is(position + Position::X, Element::Air),
            south: grid_elements.is(position + Position::Y, Element::Air),
            west: grid_elements.is(position - Position::X, Element::Air),
        });
    }
}
