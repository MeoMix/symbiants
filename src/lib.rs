mod ant;
mod background;
mod camera;
mod elements;
mod gravity;
mod map;
mod mouse;
mod name_list;
mod pancam;
mod render;
mod settings;
mod simulation;
mod time;
mod ui;
mod world_rng;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use camera::CameraPlugin;
use map::WorldMap;
use settings::Settings;
use simulation::SimulationPlugin;
use time::{IsFastForwarding, PendingTicks, DEFAULT_TICK_RATE};
use ui::{loading_text_update_system, setup_loading_text_system};
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
        .insert_resource(FixedTime::new_from_secs(DEFAULT_TICK_RATE))
        .insert_resource(IsFastForwarding(false))
        .insert_resource(PendingTicks(0))
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
        .add_startup_system(setup_loading_text_system)
        .add_system(loading_text_update_system)
        .add_plugin(CameraPlugin)
        .add_plugin(SimulationPlugin);
    }
}
