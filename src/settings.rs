use bevy::prelude::{Color, Resource};

#[derive(Clone)]
pub struct Probabilities {
    pub random_dig: f32,              // dig down while wandering
    pub random_drop: f32,             // drop while wandering
    pub random_turn: f32,             // turn while wandering
    pub below_surface_dirt_dig: f32,  // chance to dig dirt when below surface level
    pub above_surface_sand_drop: f32, // chance to randomly drop sand when at-or-above surface level
    pub below_surface_food_drop: f32, // chance to randomly drop food when below surface level

    pub above_surface_queen_nest_dig: f32,
    pub below_surface_queen_nest_dig: f32,
}

#[derive(Resource, Clone)]
pub struct Settings {
    pub auto_save_interval_ms: isize,
    pub world_width: isize,
    pub world_height: isize,
    // sand turns to dirt when stacked this high
    pub compact_sand_depth: isize,
    pub initial_dirt_percent: f32,
    pub initial_ant_worker_count: isize,
    pub ant_color: Color,
    pub probabilities: Probabilities,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            // Save the world automatically because it's possible the browser could crash so saving on window unload isn't 100% reliable.
            auto_save_interval_ms: 60_000,
            world_width: 144,
            world_height: 81,
            compact_sand_depth: 15,
            initial_dirt_percent: 3.0 / 4.0,
            initial_ant_worker_count: 200,
            ant_color: Color::rgb(0.584, 0.216, 0.859), // purple!
            probabilities: Probabilities {
                random_dig: 0.003,
                random_drop: 0.003,
                random_turn: 0.005,
                below_surface_dirt_dig: 0.10,
                above_surface_sand_drop: 0.10,
                below_surface_food_drop: 0.10,

                above_surface_queen_nest_dig: 0.10,
                below_surface_queen_nest_dig: 0.50,
            },
        }
    }
}
