mod main_menu;
mod save;
mod settings;
mod story;
mod story_state;
mod ui;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_save::{SavePlugin, SaveableRegistry};
use bevy_turborand::prelude::*;
use main_menu::update_main_menu;
use story_state::StoryState;
use ui::CoreUIPlugin;
use story::{
    camera::CameraPlugin, crater_simulation::CraterSimulationPlugin,
    nest_simulation::NestSimulationPlugin, pointer::IsPointerCaptured,
    story_time::StoryPlaybackState, ui::StoryUIPlugin,
};

pub struct SymbiantsPlugin;

impl Plugin for SymbiantsPlugin {
    fn build(&self, app: &mut App) {
        // TODO: All this stuff is common to both CraterSimulation and NestSimulation and needs to find a good common home.
        app.init_resource::<SaveableRegistry>();

        // Some resources should be available for the entire lifetime of the application.
        // For example, IsPointerCaptured is a UI resource which is useful when interacting with the GameStart menu.
        app.init_resource::<IsPointerCaptured>();
        // TODO: I put very little thought into initializing this resource always vs saving/loading the seed.
        app.init_resource::<GlobalRng>();

        app.add_state::<StoryState>();
        // TODO: call this in setup_story_time?
        app.add_state::<StoryPlaybackState>();

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // Be aggressive in preventing ambiguous systems from running in parallel to prevent unintended headaches.
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Error,
                ..default()
            });
        })
        // Only want SavePlugin not SavePlugins - just need basic snapshot logic not UI persistence or save/load methods.
        .add_plugins((RngPlugin::default(), SavePlugin))
        .add_plugins((
            CameraPlugin,
            CoreUIPlugin,
            StoryUIPlugin,
            NestSimulationPlugin,
            CraterSimulationPlugin,
        ));

        app.add_systems(
            Update,
            update_main_menu.run_if(in_state(StoryState::GatheringSettings)),
        );
    }
}
