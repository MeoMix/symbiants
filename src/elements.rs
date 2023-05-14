use super::map::{Position, WorldMap};
use bevy::{prelude::*, sprite::Anchor};
use serde::{Deserialize, Serialize};

// This is what's persisted as JSON.
#[derive(Serialize, Deserialize, Debug)]
pub struct ElementSaveState {
    pub element: Element,
    pub position: Position,
}

#[derive(Bundle)]
pub struct ElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
    position: Position,
}

#[derive(Component, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

impl ElementBundle {
    pub fn create(element: Element, position: Position) -> Self {
        if element == Element::Sand {
            ElementBundle::create_sand(position)
        } else if element == Element::Air {
            ElementBundle::create_air(position)
        } else if element == Element::Dirt {
            ElementBundle::create_dirt(position)
        } else {
            panic!("unexpected element")
        }
    }

    fn create_sand(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation,
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
        }
    }

    fn create_air(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                // Air is transparent so reveal background
                visibility: Visibility::Hidden,
                transform: Transform {
                    translation,
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

    fn create_dirt(position: Position) -> Self {
        // The view of the model position is just an inversion along the y-axis.
        let translation = Vec3::new(position.x as f32, -position.y as f32, 1.0);

        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation,
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

// Returns true if every element in `positions` matches the provided Element type.
// NOTE: This returns true if given 0 positions.
pub fn is_all_element(
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    positions: &Vec<Position>,
    search_element: Element,
) -> bool {
    positions.iter().all(|position| {
        world_map
            .elements
            .get(position)
            .and_then(|&element| elements_query.get(element).ok())
            .map_or(false, |&element| element == search_element)
    })
}

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
pub fn setup_elements(mut commands: Commands, mut world_map: ResMut<WorldMap>) {
    let element_bundles = world_map
        .initial_state
        .elements
        .iter()
        .map(|element_save_state| {
            ElementBundle::create(element_save_state.element, element_save_state.position)
        })
        .collect::<Vec<_>>();

    for element_bundle in element_bundles {
        world_map
            .elements
            .insert(element_bundle.position, commands.spawn(element_bundle).id());
    }
}
