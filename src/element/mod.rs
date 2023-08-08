use self::commands::ElementCommandsExt;
use super::map::{Position, WorldMap};
use crate::gravity::Unstable;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod ui;

// This is what's persisted as JSON.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ElementSaveState {
    pub element: Element,
    pub position: Position,
}

// TODO: I am a little suspicious about using MORE marker components to indicate more clearly the exact type of an element
// rather than relying on properties of the element, but I am going with this for now for simplicity and performance.
// As an example, say a piece of sand falls down and air moves up. I want to mark elements adjacent to the air as unstable.
// I can read the position of the sand and see it has changed, but Bevy does not provide the ability to know the previous value.
// So, I need to read the position of the air because its current value is what is useful to me.
// How do I find that air? Well, I could iterate over all elements (slow), I could iterate over all not-Unstable elements (air is never unstable, but same goes for dirt and sometimes sand/food) (also slow)
// or I could tag air with a component that indicates it is air (fast).
// Another option could be to emit an event when I move the dirt saying where it moved from.
#[derive(Component)]
pub struct Air;

#[derive(Bundle)]
pub struct AirElementBundle {
    element: Element,
    position: Position,
    air: Air,
}

impl AirElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Air,
            air: Air,
            position,
        }
    }
}

#[derive(Bundle)]
pub struct DirtElementBundle {
    element: Element,
    position: Position,
}

impl DirtElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Dirt,
            position,
        }
    }
}

#[derive(Component)]
pub struct Crushable;

#[derive(Bundle)]
pub struct SandElementBundle {
    element: Element,
    position: Position,
    crushable: Crushable,
    unstable: Unstable,
}

impl SandElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Sand,
            position,
            crushable: Crushable,
            unstable: Unstable,
        }
    }
}

#[derive(Bundle)]
pub struct FoodElementBundle {
    element: Element,
    position: Position,
    unstable: Unstable,
}

impl FoodElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            element: Element::Food,
            position,
            unstable: Unstable,
        }
    }
}

#[derive(Component, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Air,
    Dirt,
    Sand,
    Food,
}

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
pub fn setup_elements(mut commands: Commands, world_map: Res<WorldMap>) {
    for &ElementSaveState { element, position } in world_map.initial_state().elements.iter() {
        commands.spawn_element(element, position);
    }
}
