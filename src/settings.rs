use bevy::{prelude::*, reflect::Reflect};
use bevy_save::SaveableRegistry;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{common::register, world_map::position::Position};

#[derive(Clone, Copy, Reflect, Debug)]
pub struct Probabilities {
    pub random_drop: f32,             // drop while wandering
    pub random_turn: f32,             // turn while wandering
    pub random_fall: f32,             // fall while upside down
    pub random_slip: f32,             // fall while vertical
    pub above_surface_sand_drop: f32, // chance to randomly drop sand when at-or-above surface level
    pub above_surface_food_dig: f32,
    pub below_surface_food_dig: f32,
    pub below_surface_food_drop: f32, // chance to randomly drop food when below surface level
    pub below_surface_food_adjacent_food_drop: f32,
    pub above_surface_queen_food_drop: f32,
    pub above_surface_queen_nest_dig: f32,
    pub below_surface_queen_nest_dig: f32,
    pub expand_nest: f32,
    pub sleep_emote: f32,
}

#[derive(Resource, Copy, Clone, Reflect, Debug)]
#[reflect(Resource)]
pub struct Settings {
    pub snapshot_interval: isize,
    pub save_interval: isize,
    pub world_width: isize,
    pub world_height: isize,
    pub initial_dirt_percent: f32,
    pub initial_ant_worker_count: isize,
    pub ant_color: Color,
    pub chamber_size: isize,
    pub tunnel_length: isize,
    pub emote_duration: isize,
    pub probabilities: Probabilities,
}

// TODO: It feels weird to put these methods here rather than on WorldMap, but I need access to these
// calculations when creating a new WorldMap instance.
impl Settings {
    pub fn get_surface_level(&self) -> isize {
        (self.world_height as f32 - (self.world_height as f32 * self.initial_dirt_percent)) as isize
    }

    pub fn get_random_surface_position(&self, rng: &mut Mut<GlobalRng>) -> Position {
        Position::new(rng.isize(0..self.world_width), self.get_surface_level())
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            // Save the world automatically because it's possible the browser could crash so saving on window unload isn't 100% reliable.
            save_interval: 60,
            // Saving data to local storage is slow, but generating the snapshot of the world is also slow.
            // Take snapshots aggressively because browser tab closes too quickly to JIT snapshot.
            snapshot_interval: 5, // TODO: prefer 1 here but it's too slow, makes sim stutter
            world_width: 144,
            // TODO: I want this to be able to go to 400 without lag and without breaking local storage
            world_height: 144,
            initial_dirt_percent: 2.0 / 4.0,
            initial_ant_worker_count: 0,
            ant_color: Color::rgb(0.584, 0.216, 0.859), // purple!
            chamber_size: 5,
            tunnel_length: 12,
            emote_duration: 30,
            probabilities: Probabilities {
                random_drop: 0.003,
                random_turn: 0.005,
                // Ants slip/fall due to gravity when upside down or vertical.
                // These settings help prevent scenarios where ants dig themselves onto islands and become trapped.
                // If these settings are set too high then it will become difficult to haul sand out of nest.
                random_fall: 0.002,
                random_slip: 0.001,
                above_surface_sand_drop: 0.04,
                above_surface_food_dig: 0.50,
                below_surface_food_dig: 0.10,
                below_surface_food_drop: 0.10,
                below_surface_food_adjacent_food_drop: 0.50,
                above_surface_queen_food_drop: 0.50,
                above_surface_queen_nest_dig: 0.10,
                below_surface_queen_nest_dig: 0.50,
                // TODO: keep playing with this value. lower chance = more cramped nest, but less sand to manage.
                expand_nest: 0.2,
                sleep_emote: 0.001,
            },
        }
    }
}

pub fn register_settings(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Settings>(&app_type_registry, &mut saveable_registry);
    register::<Probabilities>(&app_type_registry, &mut saveable_registry);
}

pub fn pre_setup_settings(mut commands: Commands) {
    commands.init_resource::<Settings>();
}

pub fn teardown_settings(mut commands: Commands) {
    commands.remove_resource::<Settings>();
}
