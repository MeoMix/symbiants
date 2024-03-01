use crate::common::Zone;
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};

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

// Each ant maintains an internal timer that determines when it will act next.
// This adds a little realism by varying when movements occur and allows for flexibility
// in the simulation run speed.
pub fn ants_initiative<Z: Zone>(
    mut alive_ants_query: Query<&mut Initiative, With<Z>>,
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
