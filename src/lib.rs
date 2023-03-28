mod ant;
mod background;
mod camera;
mod elements;
mod gravity;
mod map;
mod render;
mod settings;
mod world_rng;

use ant::AntsPlugin;
use background::BackgroundPlugin;
use bevy::prelude::*;
use camera::CameraPlugin;
use elements::ElementsPlugin;
use gravity::GravityPlugin;
use map::WorldMap;
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use render::RenderPlugin;
use settings::Settings;
use world_rng::WorldRng;

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        // Defines the amount of time that should elapse between each physics step.
        let fixed_time = FixedTime::new_from_secs(1.0 / 60.0);
        let settings = Settings::default();
        let world_map = WorldMap::new(
            settings.world_width,
            settings.world_height,
            settings.initial_dirt_percent,
            None,
        );

        let world_rng = WorldRng {
            rng: StdRng::seed_from_u64(OsRng {}.gen()),
        };

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(fixed_time)
        .insert_resource(world_map)
        .insert_resource(settings)
        .insert_resource(world_rng)
        .add_plugin(CameraPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(ElementsPlugin)
        .add_plugin(AntsPlugin)
        .add_plugin(GravityPlugin)
        .add_plugin(RenderPlugin);
    }
}
