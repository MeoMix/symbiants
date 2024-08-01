use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_turborand::prelude::*;
use rendering::RenderingPlugin;
use simulation::SimulationPlugin;
use ui::UIPlugin;

pub struct SymbiantsPlugin;

impl Plugin for SymbiantsPlugin {
    fn build(&self, app: &mut App) {
        // Use a shared, common source of randomness so that the simulation is deterministic.
        app.init_resource::<GlobalRng>();

        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        // Ensure stuff like F5, F12, right-click to show context menu works in WASM context.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            RngPlugin::default(),
            UIPlugin,
            SimulationPlugin,
            RenderingPlugin,
        ));
    }
}
