use super::{
    Angle, AntBundle, AntColor, AntInventory, AntOrientation, AntRole, Dead, Facing, Initiative, AntName,
};
use crate::{
    grid::position::Position,
    name_list::get_random_name,
    time::{DEFAULT_TICK_RATE, SECONDS_PER_HOUR},
};
use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Birthing {
    value: f32,
    max: f32,
    rate_of_birthing: f32,
}

impl Birthing {
    pub fn default() -> Self {
        let max = 100.0;
        let rate_of_birthing = max / (SECONDS_PER_HOUR as f32 * DEFAULT_TICK_RATE);

        Self {
            value: 0.0,
            max,
            rate_of_birthing,
        }
    }

    pub fn tick(&mut self) {
        self.value = (self.value + self.rate_of_birthing).min(self.max);
    }

    pub fn is_ready(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
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
        Without<Dead>,
    >,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut birthing, position, color, orientation, mut initiative) in
        ants_birthing_query.iter_mut()
    {
        birthing.tick();

        if birthing.is_ready() && initiative.can_act() {
            let behind_position = *position + orientation.turn_around().get_forward_delta();

            // NOTE: As written, this could spawn directly into a piece of dirt/food/etc.
            // This isn't going to cause the application to panic, but isn't visually appealing, either.
            // Could introduce a custom command and prevent spawning if the tile is occupied and/or find nearest open tile
            // but since ants can get covered by sand already (when it falls on them) its low priority.
            // Spawn worker ant (TODO: egg instead)
            commands.spawn(AntBundle::new(
                behind_position,
                AntColor(color.0),
                AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName(get_random_name(&mut rng.reborrow())),
                Initiative::new(&mut rng.reborrow())
            ));

            birthing.reset();
            initiative.consume_action();
        }
    }
}
