mod ant;
mod background;
mod camera;
mod elements;
mod gravity;
mod position;
mod render;
mod settings;
use bevy::{prelude::*, utils::HashMap};

use crate::antfarm::{
    ant::AntsPlugin, background::BackgroundPlugin, camera::CameraPlugin, elements::ElementsPlugin,
    gravity::GravityPlugin,
};

use self::{position::Position, render::RenderPlugin, settings::Settings};

#[derive(Resource)]
pub struct WorldState {
    width: isize,
    height: isize,
    surface_level: isize,
}

impl WorldState {
    fn new(width: isize, height: isize, dirt_percent: f32) -> Self {
        WorldState {
            width,
            height,
            // TODO: Double-check for off-by-one here
            surface_level: (height as f32 - (height as f32 * dirt_percent)) as isize,
        }
    }
}

#[derive(Resource)]
pub struct WorldMap {
    pub elements: HashMap<Position, Entity>,
}

impl WorldMap {
    fn new() -> Self {
        WorldMap {
            elements: HashMap::default(),
        }
    }
}

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
        .insert_resource(WorldState::new(
            settings.world_width,
            settings.world_height,
            settings.initial_dirt_percent,
        ))
        .insert_resource(settings)
        .insert_resource(WorldMap::new())
        .add_plugin(CameraPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(ElementsPlugin)
        .add_plugin(AntsPlugin)
        .add_plugin(GravityPlugin)
        .add_plugin(RenderPlugin);
    }
}
