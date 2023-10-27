/// Ants might emote from time to time. This results in showing an emoji above their head.
/// TODO: Make this more flexible and invert control. For now prioritizing ease of implementation.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum EmoteType {
    // TODO: It's weird to have None as a default when I'll remove the Component entirely instead of setting to None
    #[default]
    None,
    Asleep,
    Hungry,
}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Emote {
    pub emote_type: EmoteType,
    value: f32,
    max: f32,
}

impl Emote {
    pub fn new(emote_type: EmoteType) -> Self {
        Self {
            emote_type,
            value: 0.0,
            max: 100.0,
        }
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn tick(&mut self, rate_of_emote: f32) {
        self.value = (self.value + rate_of_emote).min(self.max);
    }

    pub fn is_expired(&self) -> bool {
        self.value >= self.max
    }
}