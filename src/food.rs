use bevy::prelude::*;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct FoodCount(pub isize);
