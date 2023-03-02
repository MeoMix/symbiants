use bevy::prelude::*;

pub struct Probabilities {
    pub random_dig: f32,         // dig down while wandering
    pub random_drop: f32,        // drop while wandering
    pub random_turn: f32,        // turn while wandering
    pub below_surface_dig: f32,  // chance to dig dirt when below surface level
    pub above_surface_drop: f32, // chance to randomly drop sand when at-or-above surface level
}

#[derive(Resource)]
pub struct Settings {
    // sand turns to dirt when stacked this high
    pub compact_sand_depth: i32,
    pub initial_dirt_percent: f32,
    pub initial_ant_count: i32,
    pub ant_color: Color,
    pub probabilities: Probabilities,
}
