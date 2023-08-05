use bevy::prelude::*;

pub mod ui;

#[derive(Component, Copy, Clone)]
pub struct TranslationOffset(pub Vec3);

#[derive(Component)]
pub struct Label(pub Entity);
