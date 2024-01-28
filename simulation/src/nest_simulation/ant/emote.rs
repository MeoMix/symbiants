// TODO: move this to an area that is clearly UI-only
/// Ants might emote from time to time. This results in showing an emoji above their head.
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub enum EmoteType {
    Asleep,
    FoodLove,
}

#[derive(Component, Debug, PartialEq, Copy, Clone)]
pub struct Emote {
    emote_type: EmoteType,
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

    pub fn emote_type(&self) -> EmoteType {
        self.emote_type
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
