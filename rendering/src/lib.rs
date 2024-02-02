mod camera;
pub mod common;
mod crater_rendering;
mod nest_rendering;
pub mod pointer;

use self::camera::RenderingCameraPlugin;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use common::CommonRenderingPlugin;
use crater_rendering::CraterRenderingPlugin;
use nest_rendering::NestRenderingPlugin;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        // Keep this off to prevent spritesheet bleed at various `projection.scale` levels.
        // See https://github.com/bevyengine/bevy/issues/1949 for details.
        app.insert_resource(Msaa::Off);

        app.add_plugins((
            RenderingCameraPlugin,
            TilemapPlugin,
            CommonRenderingPlugin,
            CraterRenderingPlugin,
            NestRenderingPlugin,
        ));
    }
}
