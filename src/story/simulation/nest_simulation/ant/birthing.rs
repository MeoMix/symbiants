use crate::story::simulation::{
    common::position::Position, nest_simulation::nest::AtNest, story_time::DEFAULT_TICKS_PER_SECOND,
};

use super::{
    commands::AntCommandsExt, Angle, AntColor, AntInventory, AntName, AntOrientation, AntRole,
    Facing, Initiative,
};

use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Birthing {
    value: f32,
    max: f32,
    rate: f32,
}

impl Birthing {
    pub fn new(max_time_seconds: isize) -> Self {
        let max = 100.0;
        let rate = max / (max_time_seconds * DEFAULT_TICKS_PER_SECOND) as f32;

        Self {
            value: 0.0,
            max,
            rate,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn tick(&mut self) {
        self.value = (self.value + self.rate).min(self.max);
    }

    pub fn is_ready(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

pub fn register_birthing(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Birthing>();
}

pub fn ants_birthing(
    mut ants_birthing_query: Query<
        (
            &mut Birthing,
            &Position,
            &AntColor,
            &AntOrientation,
            &mut Initiative,
        ),
        With<AtNest>,
    >,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut birthing, position, color, orientation, mut initiative) in
        ants_birthing_query.iter_mut()
    {
        birthing.tick();

        if !initiative.can_act() {
            continue;
        }

        // Once an ant starts giving birth - they're incapacitated and cannot do anything low priority.
        initiative.consume();

        if birthing.is_ready() {
            // NOTE: As written, this could spawn directly into a piece of dirt/food/etc.
            // This isn't going to cause the application to panic, but isn't visually appealing, either.
            // Could introduce a custom command and prevent spawning if the tile is occupied and/or find nearest open tile
            // but since ants can get covered by sand already (when it falls on them) its low priority.
            // Spawn worker ant (TODO: egg instead)
            commands.spawn_ant(
                orientation.get_behind_position(position),
                AntColor(color.0),
                AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName::random(&mut rng.reborrow()),
                Initiative::new(&mut rng.reborrow()),
                AtNest,
            );

            birthing.reset();
        }
    }
}
