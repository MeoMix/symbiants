pub mod birthing;
pub mod chambering;
pub mod dig;
pub mod drop;
pub mod nest_expansion;
pub mod nesting;
pub mod sleep;
pub mod travel;
pub mod tunneling;
pub mod wander;

use std::f32::consts::PI;

use crate::common::position::Position;

use self::{birthing::Birthing, chambering::Chambering, sleep::Asleep, tunneling::Tunneling};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    // TODO: This might be nest-specific, but maybe needs to be supported at crater just in case
    app_type_registry.write().register::<Asleep>();

    app_type_registry.write().register::<NestOrientation>();
    app_type_registry.write().register::<NestFacing>();
    app_type_registry.write().register::<NestAngle>();

    // TODO: These seem nest-specific
    app_type_registry.write().register::<Birthing>();
    app_type_registry.write().register::<Tunneling>();
    app_type_registry.write().register::<Chambering>();
}


#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum NestFacing {
    #[default]
    Left,
    Right,
}

impl NestFacing {
    pub fn random(rng: &mut Mut<GlobalRng>) -> Self {
        if rng.bool() {
            NestFacing::Left
        } else {
            NestFacing::Right
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum NestAngle {
    #[default]
    Zero,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

impl NestAngle {
    pub fn as_radians(self) -> f32 {
        (self as isize as f32) * PI / 180.0
    }

    /// Rotation is a value from 0 to 3. A value of 1 is a 90 degree counter-clockwise rotation. Negative values are accepted.
    /// Examples:
    ///     rotate(0, -1); // 270
    ///     rotate(0, 1); // 90
    pub fn rotate(self, rotation: i32) -> Self {
        let angles = [
            NestAngle::Zero,
            NestAngle::Ninety,
            NestAngle::OneHundredEighty,
            NestAngle::TwoHundredSeventy,
        ];

        let rotated_index = (angles.iter().position(|&a| a == self).unwrap() as i32 - rotation)
            % angles.len() as i32;
        angles[((rotated_index + angles.len() as i32) % angles.len() as i32) as usize]
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct NestOrientation {
    facing: NestFacing,
    angle: NestAngle,
}

impl NestOrientation {
    pub fn new(facing: NestFacing, angle: NestAngle) -> Self {
        Self { facing, angle }
    }

    pub fn get_facing(&self) -> NestFacing {
        self.facing
    }

    pub fn get_angle(&self) -> NestAngle {
        self.angle
    }

    // pub fn is_horizontal(&self) -> bool {
    //     self.angle == Angle::Zero || self.angle == Angle::OneHundredEighty
    // }

    pub fn is_vertical(&self) -> bool {
        self.angle == NestAngle::Ninety || self.angle == NestAngle::TwoHundredSeventy
    }

    pub fn is_upside_down(&self) -> bool {
        self.angle == NestAngle::OneHundredEighty
    }

    pub fn is_rightside_up(&self) -> bool {
        self.angle == NestAngle::Zero
    }

    pub fn is_facing_north(&self) -> bool {
        match (self.angle, self.facing) {
            (NestAngle::Ninety, NestFacing::Right) => true,
            (NestAngle::TwoHundredSeventy, NestFacing::Left) => true,
            _ => false,
        }
    }

    pub fn is_facing_south(&self) -> bool {
        match (self.angle, self.facing) {
            (NestAngle::TwoHundredSeventy, NestFacing::Right) => true,
            (NestAngle::Ninety, NestFacing::Left) => true,
            _ => false,
        }
    }

    pub fn turn_around(&self) -> Self {
        let facing = if self.facing == NestFacing::Left {
            NestFacing::Right
        } else {
            NestFacing::Left
        };

        Self::new(facing, self.angle)
    }

    pub fn rotate_forward(&self) -> Self {
        let rotation = if self.facing == NestFacing::Left {
            -1
        } else {
            1
        };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn rotate_backward(&self) -> Self {
        let rotation = if self.facing == NestFacing::Left {
            1
        } else {
            -1
        };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    fn get_ahead_delta(&self) -> Position {
        let delta = match self.angle {
            NestAngle::Zero => Position::X,
            NestAngle::Ninety => Position::NEG_Y,
            NestAngle::OneHundredEighty => Position::NEG_X,
            NestAngle::TwoHundredSeventy => Position::Y,
        };

        if self.facing == NestFacing::Left {
            delta * Position::NEG_ONE
        } else {
            delta
        }
    }

    fn get_below_delta(&self) -> Position {
        self.rotate_forward().get_ahead_delta()
    }

    fn get_behind_delta(&self) -> Position {
        self.turn_around().get_ahead_delta()
    }

    fn get_above_delta(&self) -> Position {
        self.rotate_backward().get_ahead_delta()
    }

    /// Returns the position of the tile in front of the ant's face.
    pub fn get_ahead_position(&self, position: &Position) -> Position {
        *position + self.get_ahead_delta()
    }

    /// Returns the position of the tile below the ant's feet.
    pub fn get_below_position(&self, position: &Position) -> Position {
        *position + self.get_below_delta()
    }
    /// Returns the position of the tile behind the ant's butt.
    pub fn get_behind_position(&self, position: &Position) -> Position {
        *position + self.get_behind_delta()
    }

    /// Returns the position of the tile above the ant's head.
    pub fn get_above_position(&self, position: &Position) -> Position {
        *position + self.get_above_delta()
    }

    pub fn all_orientations() -> Vec<Self> {
        let facings = [NestFacing::Left, NestFacing::Right];
        let angles = [
            NestAngle::Zero,
            NestAngle::Ninety,
            NestAngle::OneHundredEighty,
            NestAngle::TwoHundredSeventy,
        ];
        facings
            .iter()
            .flat_map(|facing| angles.iter().map(move |angle| Self::new(*facing, *angle)))
            .collect::<Vec<_>>()
    }
}
