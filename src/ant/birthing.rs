use super::{Alive, Angle, AntBundle, AntColor, AntInventory, AntOrientation, AntRole, Facing};
use crate::{map::Position, name_list::NAMES, world_rng::WorldRng};
use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Birthing {
    value: usize,
    max: usize,
}

impl Birthing {
    pub fn default() -> Self {
        Self {
            value: 0,
            // TODO: 30 minutes expressed in frame ticks
            max: 6 * 60 * 30,
        }
    }

    pub fn try_increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn is_ready(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

pub fn ants_birthing(
    mut ants_birthing_query: Query<
        (&mut Birthing, &Position, &AntColor, &AntOrientation),
        With<Alive>,
    >,
    mut commands: Commands,
    mut world_rng: ResMut<WorldRng>,
) {
    for (mut birthing, position, color, orientation) in ants_birthing_query.iter_mut() {
        birthing.try_increment();

        if birthing.is_ready() {
            // Randomly position ant facing left or right.
            let facing = if world_rng.0.gen_bool(0.5) {
                Facing::Left
            } else {
                Facing::Right
            };

            let name: &str = NAMES[world_rng.0.gen_range(0..NAMES.len())].clone();

            let behind_position = *position + orientation.turn_around().get_forward_delta();

            // NOTE: As written, this could spawn directly into a piece of dirt/food/etc.
            // This isn't going to cause the application to panic, but isn't visually appealing, either.
            // Could introduce a custom command and prevent spawning if the tile is occupied and/or find nearest open tile
            // but since ants can get covered by sand already (when it falls on them) its low priority.
            // Spawn worker ant (TODO: egg instead)
            commands.spawn(AntBundle::new(
                behind_position,
                color.0,
                AntOrientation::new(facing, Angle::Zero),
                AntInventory(None),
                AntRole::Worker,
                name,
                &mut world_rng.0,
            ));

            birthing.reset();
        }
    }
}
