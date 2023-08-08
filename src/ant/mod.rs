use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    map::{Position, WorldMap},
    world_rng::WorldRng,
};

use self::hunger::Hunger;

use super::{element::Element, settings::Settings};
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};

pub mod birthing;
mod commands;
pub mod hunger;
pub mod walk;
pub mod act;
// TODO: maybe don't want this public?
pub mod ui;

// This is what is persisted as JSON.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AntSaveState {
    pub position: Position,
    pub color: AntColor,
    pub orientation: AntOrientation,
    pub inventory: AntInventory,
    pub role: AntRole,
    pub timer: AntTimer,
    pub name: AntName,
}

#[derive(Bundle)]
struct AntBundle {
    ant: Ant,
    position: Position,
    orientation: AntOrientation,
    role: AntRole,
    timer: AntTimer,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
    alive: Alive,
    inventory: AntInventory,
}

impl AntBundle {
    pub fn new(
        position: Position,
        color: Color,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: &str,
        mut rng: &mut StdRng,
    ) -> Self {
        AntBundle {
            ant: Ant,
            position,
            orientation,
            inventory,
            role,
            timer: AntTimer::new(&mut rng),
            name: AntName(name.to_string()),
            color: AntColor(color),
            hunger: Hunger::default(),
            alive: Alive,
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AntName(pub String);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntColor(pub Color);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntInventory(pub Option<Element>);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Alive;

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ant;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum AntRole {
    Worker,
    Queen,
}

#[derive(Bundle)]
pub struct CarryingBundle {
    sprite_bundle: SpriteBundle,
    element: Element,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntTimer(pub isize);

impl AntTimer {
    pub fn new(rng: &mut StdRng) -> Self {
        Self(rng.gen_range(3..5))
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Angle {
    Zero = 0,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

impl Angle {
    pub fn as_radians(self) -> f32 {
        (self as isize as f32) * PI / 180.0
    }

    /**
     * Rotation is a value from 0 to 3. A value of 1 is a 90 degree counter-clockwise rotation. Negative values are accepted.
     * Examples:
     *  rotate(0, -1); // 270
     *  rotate(0, 1); // 90
     */
    pub fn rotate(self, rotation: i32) -> Self {
        let angles = [
            Angle::Zero,
            Angle::Ninety,
            Angle::OneHundredEighty,
            Angle::TwoHundredSeventy,
        ];

        let rotated_index = (angles.iter().position(|&a| a == self).unwrap() as i32 - rotation)
            % angles.len() as i32;
        angles[((rotated_index + angles.len() as i32) % angles.len() as i32) as usize]
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct AntOrientation {
    facing: Facing,
    angle: Angle,
}

impl AntOrientation {
    pub fn new(facing: Facing, angle: Angle) -> Self {
        Self { facing, angle }
    }

    // Convert AntOrientation to Transform.Scale, z-index is naively set to 1 for now
    pub fn as_world_scale(&self) -> Vec3 {
        Vec3 {
            x: if self.get_facing() == Facing::Left {
                -1.0
            } else {
                1.0
            },
            y: 1.0,
            z: 1.0,
        }
    }

    pub fn as_world_rotation(&self) -> Quat {
        Quat::from_rotation_z(self.get_angle().as_radians())
    }

    pub fn get_facing(&self) -> Facing {
        self.facing
    }

    pub fn get_angle(&self) -> Angle {
        self.angle
    }

    pub fn is_horizontal(&self) -> bool {
        self.angle == Angle::Zero || self.angle == Angle::OneHundredEighty
    }

    pub fn turn_around(&self) -> Self {
        let facing = if self.facing == Facing::Left {
            Facing::Right
        } else {
            Facing::Left
        };

        Self::new(facing, self.angle)
    }

    pub fn flip_onto_back(&self) -> Self {
        self.rotate_backward().rotate_backward()
    }

    pub fn rotate_forward(&self) -> Self {
        let rotation = if self.facing == Facing::Left { -1 } else { 1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn rotate_backward(&self) -> Self {
        let rotation = if self.facing == Facing::Left { 1 } else { -1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn get_forward_delta(&self) -> Position {
        let delta = match self.angle {
            Angle::Zero => Position::X,
            Angle::Ninety => Position::NEG_Y,
            Angle::OneHundredEighty => Position::NEG_X,
            Angle::TwoHundredSeventy => Position::Y,
        };

        if self.facing == Facing::Left {
            delta * Position::NEG_ONE
        } else {
            delta
        }
    }

    pub fn all_orientations() -> Vec<Self> {
        let facings = [Facing::Left, Facing::Right];
        let angles = [
            Angle::Zero,
            Angle::Ninety,
            Angle::OneHundredEighty,
            Angle::TwoHundredSeventy,
        ];
        facings
            .iter()
            .flat_map(|facing| {
                angles
                    .iter()
                    .map(move |angle| Self::new(*facing, *angle))
            })
            .collect::<Vec<_>>()
    }
}

pub fn setup_ants(
    mut commands: Commands,
    settings: Res<Settings>,
    world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    for ant_save_state in world_map.initial_state().ants.iter() {
        commands.spawn(AntBundle::new(
            ant_save_state.position,
            settings.ant_color,
            ant_save_state.orientation,
            ant_save_state.inventory,
            ant_save_state.role,
            ant_save_state.name.0.as_str(),
            &mut world_rng.0,
        ));
    }
}

// Each ant maintains an internal timer that determines when it will act next.
// This adds a little realism by varying when movements occur and allows for flexibility
// in the simulation run speed.
pub fn ants_update_action_timer(
    mut ants_query: Query<&mut AntTimer, With<Alive>>,
    mut world_rng: ResMut<WorldRng>,
) {
    for mut timer in ants_query.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
            continue;
        }

        *timer = AntTimer::new(&mut world_rng.0);
    }
}

// TODO: tests
