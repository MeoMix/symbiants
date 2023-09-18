use super::grid::position::Position;
use crate::{
    common::{register, Id},
    gravity::Unstable,
    settings::Settings,
};
use bevy::prelude::*;
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
pub struct Crushable;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Sand;

#[derive(Bundle)]
pub struct SandElementBundle {
    id: Id,
    element: Element,
    sand: Sand,
    position: Position,
    crushable: Crushable,
    unstable: Unstable,
}

impl SandElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            id: Id::default(),
            element: Element::Sand,
            sand: Sand,
            position,
            crushable: Crushable,
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
    // TODO: IDK, I needed a default for Reflect (IDK why) but I don't necessarily feel like Air is the perfect choice?
    #[default]
    Air,
    Dirt,
    Sand,
    Food,
}

pub fn initialize_element(world: &mut World) {
    register::<Element>(world);
    register::<Air>(world);
    register::<Food>(world);
    register::<Dirt>(world);
    register::<Sand>(world);
    register::<Crushable>(world);
    register::<Unstable>(world);
}

pub fn setup_element(world: &mut World) {
    let settings = world.resource::<Settings>().clone();

    for y in 0..settings.world_height {
        for x in 0..settings.world_width {
            let position = Position::new(x, y);

            if y <= settings.get_surface_level() {
                world.spawn(AirElementBundle::new(position));
            } else {
                world.spawn(DirtElementBundle::new(position));
            }
        }
    }
}

pub fn cleanup_element(world: &mut World) {
    let mut element_query = world.query_filtered::<Entity, With<Element>>();
    let elements = element_query.iter(&world).collect::<Vec<_>>();

    for element in elements {
        world.entity_mut(element).despawn_recursive();
    }
}

pub fn deinitialize_element() {}
