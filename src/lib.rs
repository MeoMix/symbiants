mod core_ui;
mod main_camera;
mod main_menu;
mod story;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_turborand::prelude::*;
use core_ui::CoreUIPlugin;
use main_camera::MainCameraPlugin;
use main_menu::update_main_menu;

use story::{rendering::RenderingPlugin, ui::StoryUIPlugin};
use simulation::{app_state::AppState, SimulationPlugin};

pub struct SymbiantsPlugin;

impl Plugin for SymbiantsPlugin {
    fn build(&self, app: &mut App) {
        // See https://github.com/bevyengine/bevy/pull/10623 for details.
        app.insert_resource(AssetMetaCheck::Never);

        // Keep this off to prevent spritesheet bleed at various `projection.scale` levels.
        // See https://github.com/bevyengine/bevy/issues/1949 for details.
        app.insert_resource(Msaa::Off);

        // Use a shared, common source of randomness so that the simulation is deterministic.
        app.init_resource::<GlobalRng>();

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
        .add_plugins((
            RngPlugin::default(),
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
