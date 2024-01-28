mod core_ui;
mod main_camera;
mod main_menu;
mod save;
mod story;

use bevy::{
    asset::AssetMetaCheck,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_save::SavePlugin;
use bevy_turborand::prelude::*;
use core_ui::CoreUIPlugin;
use main_camera::MainCameraPlugin;
use main_menu::update_main_menu;

use story::{
    rendering::RenderingPlugin,
    simulation::{
        app_state::AppState, story_time::StoryPlaybackState, SimulationPlugin, SimulationUpdate,
    },
    ui::StoryUIPlugin,
};

pub struct SymbiantsPlugin;

impl Plugin for SymbiantsPlugin {
    fn build(&self, app: &mut App) {
        // See https://github.com/bevyengine/bevy/pull/10623 for details.
        app.insert_resource(AssetMetaCheck::Never);
        // See https://github.com/bevyengine/bevy/issues/1949 for details.
        // Keep this off to prevent spritesheet bleed at various `projection.scale` levels.
        app.insert_resource(Msaa::Off);

        app.init_resource::<GlobalRng>();

        // TODO: call this in setup_story_time?
        app.add_state::<StoryPlaybackState>();

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
        .edit_schedule(SimulationUpdate, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Error,
                ..default()
            });
        })
        // Only want SavePlugin not SavePlugins - just need basic snapshot logic not UI persistence or save/load methods.
        .add_plugins((RngPlugin::default(), SavePlugin))
        .add_plugins((
            MainCameraPlugin,
            CoreUIPlugin,
            StoryUIPlugin,
            SimulationPlugin,
            RenderingPlugin,
        ));

        app.add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::SelectStoryMode)),
        );
    }
}
