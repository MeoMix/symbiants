mod main_camera;
mod story;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_turborand::prelude::*;
use main_camera::MainCameraPlugin;

use simulation::SimulationPlugin;
use story::{rendering::RenderingPlugin, ui::UIPlugin};

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
            UIPlugin,
            SimulationPlugin,
            RenderingPlugin,
        ));
    }
}
