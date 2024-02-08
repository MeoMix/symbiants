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
                        fit_canvas_to_parent: true,
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
