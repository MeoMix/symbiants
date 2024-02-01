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
    expires_at: isize,
}

impl Emote {
    pub fn new(emote_type: EmoteType, expires_at: isize) -> Self {
        Self {
            emote_type,
            expires_at,
        }
    }

    pub fn emote_type(&self) -> EmoteType {
        self.emote_type
    }

    pub fn expires_at(&self) -> isize {
        self.expires_at
    }
}
