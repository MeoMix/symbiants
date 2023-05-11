mod ant;
mod background;
mod camera;
mod elements;
mod gravity;
mod map;
mod name_list;
mod render;
mod settings;
mod simulation;
mod world_rng;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use camera::CameraPlugin;
use map::WorldMap;
use settings::Settings;
use simulation::SimulationPlugin;
use world_rng::WorldRng;

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        // Defines the amount of time that should elapse between each physics step.
        // TODO: Naturally I would think 60FPS is preferable, but it runs way too fast. Maybe this is fine?
        .insert_resource(FixedTime::new_from_secs(10.0 / 60.0))
        .init_resource::<Settings>()
        .init_resource::<WorldRng>()
        .init_resource::<WorldMap>()
        // Be aggressive in preventing ambiguous systems from running in parallel to prevent unintended headaches.
        .edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Error,
                ..default()
            });
        })
        .add_plugin(CameraPlugin)
        .add_plugin(SimulationPlugin);
    }
}
