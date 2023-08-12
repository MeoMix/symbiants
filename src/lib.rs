mod ant;
mod background;
mod camera;
mod common;
mod element;
mod food;
mod gravity;
mod map;
mod mouse;
mod name_list;
mod pancam;
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
use simulation::SimulationPlugin;
use ui::UIPlugin;

use bevy_save::prelude::*;

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
         
        )
        // Be aggressive in preventing ambiguous systems from running in parallel to prevent unintended headaches.
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Error,
                ..default()
            });
        })
        .add_plugins((SavePlugins, CameraPlugin, UIPlugin, SimulationPlugin));
    }
}
