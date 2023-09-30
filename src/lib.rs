mod ant;
mod background;
mod camera;
mod common;
mod element;
mod gravity;
mod grid;
mod pointer;
mod pheromone;
mod name_list;
mod pancam;
mod settings;
mod simulation;
mod story_state;
mod time;
mod ui;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_save::SavePlugin;
use bevy_turborand::prelude::*;
use camera::CameraPlugin;
use simulation::SimulationPlugin;
use ui::UIPlugin;

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
        // Be aggressive in preventing ambiguous systems from running in parallel to prevent unintended headaches.
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Error,
                ..default()
            });
        })
        // Only want SavePlugin not SavePlugins - just need basic snapshot logic not UI persistence or save/load methods.
        .add_plugins((RngPlugin::default(), SavePlugin))
        .add_plugins((CameraPlugin, UIPlugin, SimulationPlugin));
    }
}
