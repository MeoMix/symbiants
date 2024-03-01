pub mod dig;
pub mod emit_pheromone;
pub mod follow_pheromone;
pub mod set_pheromone_emitter;
pub mod travel;
pub mod wander;

use crate::common::position::Position;

use self::emit_pheromone::{LeavingFood, LeavingNest};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<LeavingFood>();
    app_type_registry.write().register::<LeavingNest>();
    app_type_registry.write().register::<CraterOrientation>();
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum CraterOrientation {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

impl CraterOrientation {
    pub fn get_perpendicular(&self) -> Vec<Self> {
        match self {
            Self::Up | Self::Down => vec![Self::Left, Self::Right],
            Self::Left | Self::Right => vec![Self::Up, Self::Down],
        }
    }

    pub fn all_orientations() -> Vec<Self> {
        vec![Self::Left, Self::Right, Self::Up, Self::Down]
    }

    pub fn random(rng: &mut Mut<GlobalRng>) -> Self {
        let choices = Self::all_orientations();

        *rng.sample(&choices).unwrap()
    }

    /// Returns the position of the tile in front of the ant's face.
    pub fn get_ahead_position(&self, position: &Position) -> Position {
        let ahead_delta = match self {
            Self::Up => Position::NEG_Y,
            Self::Down => Position::Y,
            Self::Left => Position::NEG_X,
            Self::Right => Position::X,
        };

        *position + ahead_delta
    }

    pub fn get_clockwise_position(&self, position: &Position) -> Position {
        let clockwise_delta = match self {
            Self::Up => Position::X,
            Self::Down => Position::NEG_X,
            Self::Left => Position::NEG_Y,
            Self::Right => Position::Y,
        };

        *position + clockwise_delta
    }

    pub fn get_counterclockwise_position(&self, position: &Position) -> Position {
        let counterclockwise_delta = match self {
            Self::Up => Position::NEG_X,
            Self::Down => Position::X,
            Self::Left => Position::Y,
            Self::Right => Position::NEG_Y,
        };

        *position + counterclockwise_delta
    }

    pub fn rotate_clockwise(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn rotate_counterclockwise(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Left => Self::Down,
            Self::Down => Self::Right,
            Self::Right => Self::Up,
        }
    }

    pub fn turn_around(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}