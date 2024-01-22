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
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_save::SavePlugin;
use bevy_turborand::prelude::*;
use core_ui::CoreUIPlugin;
use main_menu::update_main_menu;
use story::{
    camera::CameraPlugin, crater_simulation::CraterSimulationPlugin, grid::VisibleGridState,
    rendering::RenderingPlugin, nest_simulation::NestSimulationPlugin,
    story_time::StoryPlaybackState, ui::StoryUIPlugin,
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
        .add_plugins((RngPlugin::default(), SavePlugin, TilemapPlugin))
        .add_plugins((
            CameraPlugin,
            CoreUIPlugin,
            StoryUIPlugin,
            NestSimulationPlugin,
            RenderingPlugin,
            CraterSimulationPlugin,
        ));

        app.add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::ShowMainMenu)),
        );
    }
}
