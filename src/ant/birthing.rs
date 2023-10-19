use super::{
    commands::AntCommandsExt, Angle, AntColor, AntInventory, AntName, AntOrientation, AntRole,
    Facing, Initiative,
};
use crate::{
    common::register,
    name_list::get_random_name,
    story_time::{DEFAULT_TICKS_PER_SECOND, SECONDS_PER_HOUR},
    world_map::position::Position,
};
use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Birthing {
    value: f32,
    max: f32,
}

impl Default for Birthing {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
        }
    }
}

impl Birthing {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn tick(&mut self, rate_of_birthing: f32) {
        self.value = (self.value + rate_of_birthing).min(self.max);
    }

    pub fn is_ready(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

pub fn register_birthing(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Birthing>(&app_type_registry, &mut saveable_registry);
}

pub fn ants_birthing(
    mut ants_birthing_query: Query<(
        &mut Birthing,
        &Position,
        &AntColor,
        &AntOrientation,
        &mut Initiative,
    )>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut birthing, position, color, orientation, mut initiative) in
        ants_birthing_query.iter_mut()
    {
        if !initiative.can_move() {
            continue;
        }

        // Once an ant starts giving birth - they're incapacitated and cannot move.
        initiative.consume_movement();

        // Create offspring once per full real-world hour.
        let rate_of_birthing =
            birthing.max() / (SECONDS_PER_HOUR * DEFAULT_TICKS_PER_SECOND) as f32;
        birthing.tick(rate_of_birthing);

        if birthing.is_ready() && initiative.can_act() {
            let behind_position = orientation.get_behind_position(position);

            // NOTE: As written, this could spawn directly into a piece of dirt/food/etc.
            // This isn't going to cause the application to panic, but isn't visually appealing, either.
            // Could introduce a custom command and prevent spawning if the tile is occupied and/or find nearest open tile
            // but since ants can get covered by sand already (when it falls on them) its low priority.
            // Spawn worker ant (TODO: egg instead)
            commands.spawn_ant(
                behind_position,
                AntColor(color.0),
                AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName(get_random_name(&mut rng.reborrow())),
                Initiative::new(&mut rng.reborrow()),
            );

            birthing.reset();
            initiative.consume();
        }
    }
}
