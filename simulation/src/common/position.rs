use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

#[derive(
    Component,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Copy,
    Clone,
    Reflect,
    Default,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
#[reflect(Component)]
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

    /// Returns the Manhattan Distance between two positions
    pub fn distance(&self, other: &Position) -> isize {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    // Returns all positions adjacent to this position. May include out-of-bounds positions.
    pub fn get_adjacent_positions(&self) -> Vec<Position> {
        vec![
            *self + Self::X,
            *self + Self::NEG_X,
            *self + Self::Y,
            *self + Self::NEG_Y,
        ]
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
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
