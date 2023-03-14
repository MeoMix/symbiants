mod ant;
mod background;
mod camera;
mod elements;
mod gravity;
mod map;
mod render;
mod settings;
use bevy::prelude::*;

use crate::antfarm::{
    ant::AntsPlugin, background::BackgroundPlugin, camera::CameraPlugin, elements::ElementsPlugin,
    gravity::GravityPlugin,
};

use self::{
    map::{Position, WorldMap},
    render::RenderPlugin,
    settings::Settings,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: I've declared const here to ensure they are accessed as resources

        let settings = Settings::default();

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .insert_resource(WorldMap::new(
            settings.world_width,
            settings.world_height,
            settings.initial_dirt_percent,
            None,
        ))
        .insert_resource(settings)
        .add_plugin(CameraPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(ElementsPlugin)
        .add_plugin(AntsPlugin)
        .add_plugin(GravityPlugin)
        .add_plugin(RenderPlugin);
    }
}
