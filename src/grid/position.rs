use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

use super::WorldMap;

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
    pub fn as_world_position(&self, world_map: &Res<WorldMap>) -> Vec3 {
        let y_offset = *world_map.height() as f32 / 2.0;
        let x_offset = *world_map.width() as f32 / 2.0;

        Vec3 {
            // NOTE: unit width is 1.0 so add 0.5 to center the position
            x: self.x as f32 - x_offset + 0.5,
            // The view of the model position is just an inversion along the y-axis.
            y: -self.y as f32 + y_offset - 0.5,
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
