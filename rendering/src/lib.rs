pub mod common;
mod crater;
mod nest;

use bevy::prelude::*;
use common::CommonRenderingPlugin;
use crater::CraterRenderingPlugin;
use nest::NestRenderingPlugin;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Keep this off to prevent spritesheet bleed at various `projection.scale` levels.
        // See https://github.com/bevyengine/bevy/issues/1949 for details.
        app.insert_resource(Msaa::Off);

        app.add_plugins((
            CommonRenderingPlugin,
            CraterRenderingPlugin,
            NestRenderingPlugin,
        ));
    }
}
