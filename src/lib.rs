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

        // See https://github.com/bevyengine/bevy/pull/10623 for details.
        app.insert_resource(AssetMetaCheck::Never);

        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // TODO: re-enable this and delete canvas CSS when Bevy 0.13.1 or later is released.
                        // fit_canvas_to_parent: true,
                        // Ensure stuff like F5, F12, right-click to show context menu works in WASM context.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
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
