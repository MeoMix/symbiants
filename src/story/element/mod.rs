use crate::{
    settings::Settings,
    story::{
        common::{position::Position, register, Id},
        // TODO: Prefer not to couple to Gravity
        nest_simulation::gravity::Unstable,
    },
};
use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod ui;

// TODO: I am a little suspicious about using MORE marker components to indicate more clearly the exact type of an element
// rather than relying on properties of the element, but I am going with this for now for simplicity and performance.
// As an example, say a piece of sand falls down and air moves up. I want to mark elements adjacent to the air as unstable.
// I can read the position of the sand and see it has changed, but Bevy does not provide the ability to know the previous value.
// So, I need to read the position of the air because its current value is what is useful to me.
// How do I find that air? Well, I could iterate over all elements (slow), I could iterate over all not-Unstable elements (air is never unstable, but same goes for dirt and sometimes sand/food) (also slow)
// or I could tag air with a component that indicates it is air (fast).
// Another option could be to emit an event when I move the dirt saying where it moved from.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Air;

#[derive(Bundle)]
pub struct AirElementBundle {
    id: Id,
    element: Element,
    position: Position,
    air: Air,
}

impl AirElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            id: Id::default(),
            element: Element::Air,
            air: Air,
            position,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Dirt;

#[derive(Bundle)]
pub struct DirtElementBundle {
    id: Id,
    element: Element,
    dirt: Dirt,
    position: Position,
}

impl DirtElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            id: Id::default(),
            element: Element::Dirt,
            dirt: Dirt,
            position,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Sand;

#[derive(Bundle)]
pub struct SandElementBundle {
    id: Id,
    element: Element,
    sand: Sand,
    position: Position,
    unstable: Unstable,
}

impl SandElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            id: Id::default(),
            element: Element::Sand,
            sand: Sand,
            position,
            unstable: Unstable,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Food;

#[derive(Bundle)]
pub struct FoodElementBundle {
    id: Id,
    element: Element,
    food: Food,
    position: Position,
    unstable: Unstable,
}

impl FoodElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            id: Id::default(),
            element: Element::Food,
            food: Food,
            position,
            unstable: Unstable,
        }
    }
}

#[derive(Component, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, Reflect, Default)]
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

pub fn register_element(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Element>(&app_type_registry, &mut saveable_registry);
    register::<Air>(&app_type_registry, &mut saveable_registry);
    register::<Food>(&app_type_registry, &mut saveable_registry);
    register::<Dirt>(&app_type_registry, &mut saveable_registry);
    register::<Sand>(&app_type_registry, &mut saveable_registry);
    register::<Unstable>(&app_type_registry, &mut saveable_registry);
}

pub fn setup_element(settings: Res<Settings>, mut commands: Commands) {
    for y in 0..settings.nest_height {
        for x in 0..settings.nest_width {
            let position = Position::new(x, y);

            // FIXME: These should be commands.spawn_element but need to fix circularity with expecting Nest to exist.
            if y <= settings.get_surface_level() {
                commands.spawn(AirElementBundle::new(position));
            } else {
                commands.spawn(DirtElementBundle::new(position));
            }
        }
    }
}

pub fn teardown_element(mut commands: Commands, element_query: Query<Entity, With<Element>>) {
    for element_entity in element_query.iter() {
        commands.entity(element_entity).despawn_recursive();
    }
}