pub mod commands;
pub mod death;
pub mod digestion;
pub mod hunger;
pub mod initiative;
// pub mod sleep;
mod name_list;

use self::{digestion::Digestion, hunger::Hunger, initiative::Initiative, name_list::get_random_name};
use crate::common::{element::Element, position::Position, Zone};
use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Bundle)]
pub struct AntBundle<Z>
where
    Z: Zone,
{
    ant: Ant,
    position: Position,
    role: AntRole,
    initiative: Initiative,
    name: AntName,
    color: AntColor,
    hunger: Hunger,
    digestion: Digestion,
    inventory: AntInventory,
    zone: Z,
}

impl<Z: Zone> AntBundle<Z> {
    pub fn new(
        position: Position,
        color: AntColor,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
        zone: Z,
        // TODO: maybe these should be inserted onto entity via system afterward? otherwise constructor will grow indefinitely
        hunger: Hunger,
        digestion: Digestion,
    ) -> Self {
        Self {
            ant: Ant,
            // Queen always spawns in the center. She'll fall from the sky in the future.
            position,
            color,
            inventory,
            role,
            name,
            initiative,
            zone,
            hunger,
            digestion,
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntName(pub String);

impl AntName {
    pub fn random(rng: &mut Mut<GlobalRng>) -> Self {
        AntName(get_random_name(&mut rng.reborrow()))
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntColor(pub Color);

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component, MapEntities)]
pub struct AntInventory(pub Option<Entity>);

impl MapEntities for AntInventory {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        if let Some(entity) = self.0 {
            self.0 = Some(entity_mapper.get_or_reserve(entity));
        }
    }
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Dead;

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Ant;

impl Ant {}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum AntRole {
    #[default]
    Worker,
    Queen,
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryItem;

#[derive(Bundle)]
pub struct InventoryItemBundle<Z>
where
    Z: Zone,
{
    element: Element,
    inventory_item: InventoryItem,
    zone: Z,
}

impl<Z: Zone> InventoryItemBundle<Z> {
    pub fn new(element: Element, zone: Z) -> Self {
        InventoryItemBundle {
            element,
            inventory_item: InventoryItem,
            zone,
        }
    }
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

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Ant>();
    app_type_registry.write().register::<AntName>();
    app_type_registry.write().register::<AntColor>();
    app_type_registry.write().register::<Initiative>();
    app_type_registry.write().register::<AntRole>();
    app_type_registry.write().register::<AntInventory>();
    app_type_registry.write().register::<InventoryItem>();

    app_type_registry.write().register::<Dead>();
    app_type_registry.write().register::<Hunger>();
    app_type_registry.write().register::<Digestion>();

    // TODO: This might be nest-specific, but maybe needs to be supported at crater just in case
    // app_type_registry.write().register::<Asleep>();
}

// TODO: tests
