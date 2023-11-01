use super::{hunger::Hunger, Dead};
use crate::story_time::DEFAULT_TICKS_PER_SECOND;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Digestion {
    // TODO: Figure out interface
    pub value: f32,
    max: f32,
    rate: f32,
}

impl Digestion {
    pub fn new(max_time_seconds: isize) -> Self {
        let max = 100.0;
        let rate = max / (max_time_seconds * DEFAULT_TICKS_PER_SECOND) as f32;

        Self {
            // Start 100% digested
            value: 100.0,
            max,
            rate,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn increment(&mut self, percent: f32) {
        self.value += (self.max() * percent).min(self.value());
    }

    pub fn tick(&mut self) -> f32 {
        let new_value = (self.value + self.rate).min(self.max);
        let change = new_value - self.value;
        self.value = new_value;
        change
    }
}

// TODO: This is (relatively) expensive to perform for the amount of value the concept adds to the user.
/// Each tick, each ant processes any food it has inside of itself.
pub fn ants_digestion(mut ants_digestion_query: Query<(&mut Digestion, &mut Hunger), Without<Dead>>) {
    for (mut digestion, mut hunger) in ants_digestion_query.iter_mut() {
        let digestion_amount = digestion.tick();

        if digestion_amount > 0.0 {
            let value = hunger.value() - digestion_amount;
            hunger.set_value(value);
        }
    }
}
