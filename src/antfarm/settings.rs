use bevy::prelude::{Color, Resource};

pub struct Probabilities {
    pub random_dig: f32,         // dig down while wandering
    pub random_drop: f32,        // drop while wandering
    pub random_turn: f32,        // turn while wandering
    pub below_surface_dig: f32,  // chance to dig dirt when below surface level
    pub above_surface_drop: f32, // chance to randomly drop sand when at-or-above surface level
}

#[derive(Resource)]
pub struct Settings {
    pub world_width: isize,
    pub world_height: isize,
    // sand turns to dirt when stacked this high
    pub compact_sand_depth: isize,
    pub initial_dirt_percent: f32,
    pub initial_ant_count: isize,
    pub ant_color: Color,
    pub probabilities: Probabilities,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            world_width: 144,
            world_height: 81,
            compact_sand_depth: 15,
            initial_dirt_percent: 3.0 / 4.0,
            initial_ant_count: 20,
            ant_color: Color::rgb(0.584, 0.216, 0.859), // purple!
            probabilities: Probabilities {
                random_dig: 0.003,
                random_drop: 0.003,
                random_turn: 0.005,
                below_surface_dig: 0.10,
                above_surface_drop: 0.10,
            },
        }
    }
}
