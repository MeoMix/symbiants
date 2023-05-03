use bevy::{prelude::*, utils::HashMap};
use std::ops::{Add, Mul};

// TODO: maybe introduce a Tile concept?
#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    #[allow(dead_code)]
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const ONE: Self = Self::new(1, 1);
    pub const NEG_ONE: Self = Self::new(-1, -1);

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

impl Mul for Position {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

#[derive(Resource)]
pub struct WorldMap {
    width: isize,
    height: isize,
    surface_level: isize,
    // TODO: Should not have this be public
    pub elements: HashMap<Position, Entity>,
}

impl WorldMap {
    pub fn width(&self) -> &isize {
        &self.width
    }

    pub fn height(&self) -> &isize {
        &self.height
    }

    pub fn surface_level(&self) -> &isize {
        &self.surface_level
    }

    pub fn new(
        width: isize,
        height: isize,
        // TODO: maybe pass in surface_level rather than calculating from dirt_percent since
        // there's an implicit relationship between elements and dirt_percent (makes no sense during testing)
        dirt_percent: f32,
        elements: Option<HashMap<Position, Entity>>,
    ) -> Self {
        WorldMap {
            width,
            height,
            // TODO: Double-check for off-by-one here
            surface_level: (height as f32 - (height as f32 * dirt_percent)) as isize,
            elements: elements.unwrap_or_default(),
        }
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }
}
