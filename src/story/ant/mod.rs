use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::story::common::position::Position;

use self::{
    birthing::Birthing, chambering::Chambering, digestion::Digestion, hunger::Hunger,
    name_list::get_random_name, sleep::Asleep, tunneling::Tunneling,
};

use super::{common::{Zone, ui::ModelViewEntityMap}, element::Element, nest_simulation::nest::AtNest};
use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};

pub mod birthing;
pub mod chambering;
pub mod commands;
pub mod death;
pub mod dig;
pub mod digestion;
pub mod drop;
pub mod emote;
pub mod hunger;
mod name_list;
pub mod nest_expansion;
pub mod nesting;
pub mod sleep;
pub mod tunneling;
pub mod ui;
pub mod walk;

#[derive(Bundle)]
pub struct AntBundle<Z>
where
    Z: Zone,
{
    ant: Ant,
    position: Position,
    orientation: AntOrientation,
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
        orientation: AntOrientation,
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
            orientation,
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
pub struct InventoryItemBundle {
    element: Element,
    inventory_item: InventoryItem,
}

impl InventoryItemBundle {
    pub fn new(element: Element) -> Self {
        InventoryItemBundle {
            element,
            inventory_item: InventoryItem,
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
    pub fn new(rng: &mut Mut<GlobalRng>) -> Self {
        Self {
            has_action: false,
            has_movement: false,
            timer: rng.isize(3..5),
        }
    }

    pub fn can_move(&self) -> bool {
        self.timer == 0 && self.has_movement
    }

    pub fn can_act(&self) -> bool {
        self.timer == 0 && self.has_action
    }

    pub fn consume(&mut self) {
        self.consume_action();

        if self.can_move() {
            self.consume_movement();
        }
    }

    pub fn consume_movement(&mut self) {
        if !self.has_movement {
            panic!("Movement already consumed.")
        }

        self.has_movement = false;
    }

    /// This is very intentionally kept private. Movement must be consumed with action.
    /// Otherwise, systems lose their "source of truth" as to whether actions or movements occur first.
    fn consume_action(&mut self) {
        if !self.has_action {
            panic!("Action already consumed.")
        }

        self.has_action = false;
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum Facing {
    #[default]
    Left,
    Right,
}

impl Facing {
    pub fn random(rng: &mut Mut<GlobalRng>) -> Self {
        if rng.bool() {
            Facing::Left
        } else {
            Facing::Right
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum Angle {
    #[default]
    Zero,
    Ninety = 90,
    OneHundredEighty = 180,
    TwoHundredSeventy = 270,
}

impl Angle {
    pub fn as_radians(self) -> f32 {
        (self as isize as f32) * PI / 180.0
    }

    /// Rotation is a value from 0 to 3. A value of 1 is a 90 degree counter-clockwise rotation. Negative values are accepted.
    /// Examples:
    ///     rotate(0, -1); // 270
    ///     rotate(0, 1); // 90
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

    // pub fn is_horizontal(&self) -> bool {
    //     self.angle == Angle::Zero || self.angle == Angle::OneHundredEighty
    // }

    pub fn is_vertical(&self) -> bool {
        self.angle == Angle::Ninety || self.angle == Angle::TwoHundredSeventy
    }

    pub fn is_upside_down(&self) -> bool {
        self.angle == Angle::OneHundredEighty
    }

    pub fn is_rightside_up(&self) -> bool {
        self.angle == Angle::Zero
    }

    pub fn is_facing_north(&self) -> bool {
        match (self.angle, self.facing) {
            (Angle::Ninety, Facing::Right) => true,
            (Angle::TwoHundredSeventy, Facing::Left) => true,
            _ => false,
        }
    }

    // pub fn is_facing_south(&self) -> bool {
    //     match (self.angle, self.facing) {
    //         (Angle::TwoHundredSeventy, Facing::Right) => true,
    //         (Angle::Ninety, Facing::Left) => true,
    //         _ => false,
    //     }
    // }

    pub fn turn_around(&self) -> Self {
        let facing = if self.facing == Facing::Left {
            Facing::Right
        } else {
            Facing::Left
        };

        Self::new(facing, self.angle)
    }

    pub fn rotate_forward(&self) -> Self {
        let rotation = if self.facing == Facing::Left { -1 } else { 1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    pub fn rotate_backward(&self) -> Self {
        let rotation = if self.facing == Facing::Left { 1 } else { -1 };

        Self::new(self.facing, self.angle.rotate(rotation))
    }

    fn get_ahead_delta(&self) -> Position {
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
    mut alive_ants_query: Query<&mut Initiative, With<AtNest>>,
    mut rng: ResMut<GlobalRng>,
) {
    for mut initiative in alive_ants_query.iter_mut() {
        if initiative.timer > 0 {
            initiative.timer -= 1;

            if initiative.timer == 0 {
                initiative.has_action = true;
                initiative.has_movement = true;
            }

            continue;
        }

        *initiative = Initiative::new(&mut rng.reborrow());
    }
}

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Ant>();
    app_type_registry.write().register::<AntName>();
    app_type_registry.write().register::<AntColor>();
    app_type_registry.write().register::<Dead>();
    app_type_registry.write().register::<Asleep>();
    app_type_registry.write().register::<Initiative>();
    app_type_registry.write().register::<AntOrientation>();
    app_type_registry.write().register::<Facing>();
    app_type_registry.write().register::<Angle>();
    app_type_registry.write().register::<AntRole>();
    app_type_registry.write().register::<Hunger>();
    app_type_registry.write().register::<Digestion>();
    app_type_registry.write().register::<AntInventory>();
    app_type_registry.write().register::<InventoryItem>();
    app_type_registry.write().register::<Birthing>();
    app_type_registry.write().register::<Tunneling>();
    app_type_registry.write().register::<Chambering>();
}

// TODO: It's weird that this handles both view and model cleanup.
// Can't easily rely on UI to handle view cleanup because it only runs when AppState is TellStory.
// If its changed to run during Cleanup then resources etc might be missing.
pub fn teardown_ant(
    ant_model_query: Query<Entity, With<Ant>>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    for ant_model_entity in ant_model_query.iter() {
        if let Some(&ant_view_entity) = model_view_entity_map.0.get(&ant_model_entity) {
            commands.entity(ant_view_entity).despawn_recursive();
            model_view_entity_map.0.remove(&ant_model_entity);
        }

        commands.entity(ant_model_entity).despawn();
    }
}

// TODO: tests
