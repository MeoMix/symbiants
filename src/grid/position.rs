use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(
    Component, Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize, Reflect, Default,
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

    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn as_world_position(&self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            // The view of the model position is just an inversion along the y-axis.
            y: -self.y as f32,
            z: 1.0,
        }
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

impl Mul for Position {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}
