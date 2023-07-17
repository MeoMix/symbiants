use bevy::prelude::*;

#[derive(Resource)]
pub struct FoodCount(pub isize);

impl Default for FoodCount {
    fn default() -> Self {
        Self(0)
    }
}
