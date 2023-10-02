use bevy_save::SaveableRegistry;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    common::{register, Id},
    name_list::get_random_name,
    settings::Settings,
    world_map::position::Position,
};

use self::{
    birthing::Birthing, chambering::Chambering, hunger::Hunger, nesting::Nesting,
    tunneling::Tunneling,
};

use super::element::Element;
use bevy::prelude::*;

pub mod act;
pub mod birthing;
pub mod chambering;
pub mod commands;
pub mod hunger;
pub mod nest_expansion;
pub mod nesting;
pub mod tunneling;
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
        color: AntColor,
        orientation: AntOrientation,
        inventory: AntInventory,
        role: AntRole,
        name: AntName,
        initiative: Initiative,
    ) -> Self {
        AntBundle {
            id: Id::default(),
            ant: Ant,
            position,
            orientation,
            inventory,
            role,
            initiative,
            name,
            color,
            hunger: Hunger::default(),
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AntName(pub String);

#[derive(Component)]
pub struct AntLabel(pub Entity);

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
            inventory_item: InventoryItem { parent_id },
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
        self.has_action = false;
        self.has_movement = false;
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
            (Angle::Ninety, Facing::Left) => true,
            (Angle::TwoHundredSeventy, Facing::Right) => true,
            _ => false,
        }
    }

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
    mut ants_query: Query<&mut Initiative, Without<Dead>>,
    mut rng: ResMut<GlobalRng>,
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

        *initiative = Initiative::new(&mut rng.reborrow());
    }
}

pub fn register_ant(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Ant>(&app_type_registry, &mut saveable_registry);
    register::<AntName>(&app_type_registry, &mut saveable_registry);
    register::<AntColor>(&app_type_registry, &mut saveable_registry);
    register::<Dead>(&app_type_registry, &mut saveable_registry);
    register::<Initiative>(&app_type_registry, &mut saveable_registry);
    register::<AntOrientation>(&app_type_registry, &mut saveable_registry);
    register::<Facing>(&app_type_registry, &mut saveable_registry);
    register::<Angle>(&app_type_registry, &mut saveable_registry);
    register::<AntRole>(&app_type_registry, &mut saveable_registry);
    register::<Hunger>(&app_type_registry, &mut saveable_registry);
    register::<AntInventory>(&app_type_registry, &mut saveable_registry);
    register::<InventoryItem>(&app_type_registry, &mut saveable_registry);
    register::<Birthing>(&app_type_registry, &mut saveable_registry);
    register::<Tunneling>(&app_type_registry, &mut saveable_registry);
    register::<Chambering>(&app_type_registry, &mut saveable_registry);
}

pub fn setup_ant(settings: Res<Settings>, mut rng: ResMut<GlobalRng>, mut commands: Commands) {
    let mut rng = rng.reborrow();

    let queen_ant = AntBundle::new(
        settings.get_random_surface_position(&mut rng),
        AntColor(settings.ant_color),
        AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
        AntInventory::default(),
        AntRole::Queen,
        AntName(String::from("Queen")),
        Initiative::new(&mut rng),
    );

    // Newly created queens instinctively start building a nest.
    commands.spawn((queen_ant, Nesting::default()));

    let worker_ants = (0..settings.initial_ant_worker_count)
        .map(|_| {
            AntBundle::new(
                settings.get_random_surface_position(&mut rng),
                AntColor(settings.ant_color),
                AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName(get_random_name(&mut rng)),
                Initiative::new(&mut rng),
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(worker_ants);
}

pub fn teardown_ant(
    label_query: Query<Entity, With<AntLabel>>,
    ant_query: Query<Entity, With<Ant>>,
    mut commands: Commands,
) {
    // NOTE: labels aren't directly tied to their ants and so aren't despawned when ants are despawned.
    // This is because label should not rotate with ants and its much simpler to keep them detached to achieve this.
    for entity in label_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for ant in ant_query.iter() {
        commands.entity(ant).despawn_recursive();
    }
}

// TODO: tests
