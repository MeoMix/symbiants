use crate::gravity::Unstable;

use super::map::{Position, WorldMap};
use bevy::{prelude::*, sprite::Anchor};
use serde::{Deserialize, Serialize};

// This is what's persisted as JSON.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ElementSaveState {
    pub element: Element,
    pub position: Position,
}

#[derive(Bundle)]
pub struct AirElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    position: Position,
}

impl AirElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                // Air is transparent so reveal background
                visibility: Visibility::Hidden,
                transform: Transform {
                    translation: position.as_world_position(),
                    ..default()
                },
                sprite: Sprite {
                    // Fully transparent color - could in theory set to something if air was made visible.
                    color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Air,
            position,
        }
    }
}

#[derive(Bundle)]
pub struct DirtElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    position: Position,
}

impl DirtElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.as_world_position(),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.514, 0.396, 0.224),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Dirt,
            position,
        }
    }
}

#[derive(Component)]
pub struct Crushable;

#[derive(Bundle)]
pub struct SandElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    position: Position,
    crushable: Crushable,
    unstable: Unstable,
}

impl SandElementBundle {
    pub fn new(position: Position) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: position.as_world_position(),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgb(0.761, 0.698, 0.502),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                ..default()
            },
            element: Element::Sand,
            position,
            crushable: Crushable,
            unstable: Unstable,
        }
    }
}

#[derive(Component, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

// Returns true if every element in `positions` matches the provided Element type.
// NOTE: This returns true if given 0 positions.
pub fn is_all_element(
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    positions: &[Position],
    search_element: &Element,
) -> bool {
    positions.iter().all(|position| {
        world_map.get_element(*position).map_or(false, |&element| {
            elements_query
                .get(element)
                .map_or(false, |queried_element| *queried_element == *search_element)
        })
    })
}

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
pub fn setup_elements(mut commands: Commands, mut world_map: ResMut<WorldMap>) {
    let elements_data = world_map
        .initial_state
        .elements
        .iter()
        .map(|&ElementSaveState { element, position }| {
            let id = match element {
                Element::Air => commands.spawn(AirElementBundle::new(position)).id(),
                Element::Dirt => commands.spawn(DirtElementBundle::new(position)).id(),
                // TODO: not all sand is unstable just because it was recently loaded from save state.
                Element::Sand => commands.spawn(SandElementBundle::new(position)).id(),
            };

            (position, id)
        })
        .collect::<Vec<_>>();

    // TODO: might be able to do this quicker now that it's 2D vector
    for (position, id) in elements_data {
        world_map.set_element(position, id);
    }
}
