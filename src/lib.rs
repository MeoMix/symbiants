mod app_state;
mod core_ui;
mod main_menu;
mod save;
mod settings;
mod story;

use app_state::AppState;
use bevy::{
    asset::AssetMetaCheck,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_save::SavePlugin;
use bevy_turborand::prelude::*;
use core_ui::CoreUIPlugin;
use main_menu::update_main_menu;
use save::CompressedWebStorageBackend;
use story::{
    camera::CameraPlugin, crater_simulation::CraterSimulationPlugin, grid::VisibleGridState,
    nest_simulation::NestSimulationPlugin, story_time::StoryPlaybackState, ui::StoryUIPlugin,
};

pub struct SymbiantsPlugin;

impl Plugin for SymbiantsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetMetaCheck::Never);
        app.init_resource::<GlobalRng>();

        // TODO: better spot for this?
        app.init_resource::<CompressedWebStorageBackend>();

        app.add_state::<AppState>();
        // TODO: call this in setup_story_time?
        app.add_state::<StoryPlaybackState>();
        app.add_state::<VisibleGridState>();

        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        // NOTE: This isn't supported with Wayland.
                        // mode: WindowMode::SizedFullscreen,
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
            update_main_menu.run_if(in_state(AppState::ShowMainMenu)),
        );
    }
}
