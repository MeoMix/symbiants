use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
use std::{fmt, ops::Add};

use super::WorldState;

// TODO: maybe introduce a Tile concept?
#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Position {
    // (0,0) is the top-left corner of the viewport so these values can be represented unsigned
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

impl Add for Position {
    type Output = Self;

    // TODO: Hexx uses const_add here?
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Component)]
pub struct WorldMap {
    pub elements: HashMap<Position, Entity>,
}

// AffectedByGravity is just applied to Sand at the moment.
// It is surprisingly necessary to avoid overlapping queries in gravity system.
#[derive(Component)]
pub struct AffectedByGravity;

#[derive(Bundle)]
pub struct ElementBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
}

#[derive(Component, PartialEq, Copy, Clone, Debug)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ElementBundle {
    pub fn create_sand(translation: Vec3) -> Self {
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
        }
    }

    pub fn create_air(translation: Vec3) -> Self {
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
        }
    }

    pub fn create_dirt(translation: Vec3) -> Self {
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
        }
    }
}

// Spawn interactive elements - air/dirt/sand. Air isn't visible, background is revealed in its place.
pub fn setup_elements(parent: &mut ChildBuilder, world_state: &Res<WorldState>) {
    // TODO: probably better to create this all at once rather than spamming inserts
    let mut elements = HashMap::<Position, Entity>::new();

    // Test Sand
    let sand_bundles = (0..1).flat_map(|row_index| {
        (0..world_state.width).map(move |column_index| {
            (
                ElementBundle::create_sand(
                    // NOTE: row_index goes negative because 0,0 is top-left corner
                    Vec3::new(column_index as f32, -(row_index as f32), 1.0),
                ),
                Position {
                    x: column_index,
                    y: row_index,
                },
                AffectedByGravity,
            )
        })
    });

    for sand_bundle in sand_bundles {
        let position = sand_bundle.1;
        elements.insert(position, parent.spawn(sand_bundle).id());
    }

    // Air & Dirt
    // NOTE: starting at 1 to skip sand
    let air_bundles = (1..(world_state.surface_level + 1)).flat_map(|row_index| {
        (0..world_state.width).map(move |column_index| {
            (
                // NOTE: row_index goes negative because 0,0 is top-left corner
                ElementBundle::create_air(Vec3::new(column_index as f32, -(row_index as f32), 1.0)),
                Position {
                    x: column_index,
                    y: row_index,
                },
            )
        })
    });

    for air_bundle in air_bundles {
        let position = air_bundle.1;
        elements.insert(position, parent.spawn(air_bundle).id());
    }

    let dirt_bundles =
        ((world_state.surface_level + 1)..world_state.height).flat_map(|row_index| {
            (0..world_state.width).map(move |column_index| {
                (
                    ElementBundle::create_dirt(Vec3::new(
                        column_index as f32,
                        // NOTE: row_index goes negative because 0,0 is top-left corner
                        -(row_index as f32),
                        1.0,
                    )),
                    Position {
                        x: column_index,
                        y: row_index,
                    },
                )
            })
        });

    for dirt_bundle in dirt_bundles {
        let position = dirt_bundle.1;
        elements.insert(position, parent.spawn(dirt_bundle).id());
    }

    // TODO: Will need to sort out how to add ants
    parent.spawn(WorldMap { elements });
}
