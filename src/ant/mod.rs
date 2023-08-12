use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    map::Position,
    world_rng::WorldRng, common::Id,
};

use self::hunger::Hunger;

use super::element::Element;
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};

pub mod act;
pub mod birthing;
mod commands;
pub mod hunger;
pub mod ui;
pub mod walk;

#[derive(Bundle)]
pub struct AntBundle {
    id: Id,
    ant: Ant,
    position: Position,
    orientation: AntOrientation,
    role: AntRole,
    initiative: Initiative,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
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
            id: Id::default(),
            ant: Ant,
            position,
            orientation,
            inventory,
            role,
            initiative: Initiative::new(&mut rng),
            name: AntName(name.to_string()),
            color: AntColor(color),
            hunger: Hunger::default(),
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntName(pub String);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntColor(pub Color);

// TODO: GUID doesn't implement Copy so I've got .clone's() everywhere cuz I'm lazy
#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntInventory(pub Option<Id>);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Dead;

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Ant;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum AntRole {
    #[default] Worker,
    Queen,
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryItem { 
    pub parent_id: Id,
}

#[derive(Bundle)]
pub struct InventoryItemBundle {
    id: Id,
    element: Element,
    inventory_item: InventoryItem,
}

impl InventoryItemBundle {
    pub fn new(element: Element, parent_id: Id) -> Self {
        InventoryItemBundle {
            id: Id::default(),
            element,
            inventory_item: InventoryItem {
                parent_id
            },
        }
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Initiative {
    has_action: bool,
    has_movement: bool,
    timer: isize,
}

impl Initiative {
    pub fn new(rng: &mut StdRng) -> Self {
        Self {
            has_action: false,
            has_movement: false,
            timer: rng.gen_range(3..5),
        }
    }

    pub fn can_move(&self) -> bool {
        self.timer == 0 && self.has_movement
    }

    pub fn can_act(&self) -> bool {
        self.timer == 0 && self.has_action
    }

    pub fn consume_action(&mut self) {
        self.has_action = false;
    }

    pub fn consume_movement(&mut self) {
        self.has_movement = false;
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum Facing {
    #[default] Left,
    Right,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum Angle {
    #[default] Zero,
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

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
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
            .flat_map(|facing| angles.iter().map(move |angle| Self::new(*facing, *angle)))
            .collect::<Vec<_>>()
    }
}

// Each ant maintains an internal timer that determines when it will act next.
// This adds a little realism by varying when movements occur and allows for flexibility
// in the simulation run speed.
pub fn ants_initiative(
    mut ants_query: Query<&mut Initiative, Without<Dead>>,
    mut world_rng: ResMut<WorldRng>,
) {
    for mut initiative in ants_query.iter_mut() {
        if initiative.timer > 0 {
            initiative.timer -= 1;

            if initiative.timer == 0 {
                initiative.has_action = true;
                initiative.has_movement = true;
            }

            continue;
        }

        *initiative = Initiative::new(&mut world_rng.0);
    }
}

// TODO: tests
