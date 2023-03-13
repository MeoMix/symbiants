mod ant;
mod background;
mod elements;
mod gravity;
mod root;
mod settings;
use bevy::{prelude::*, utils::HashMap};
use std::ops::Add;

use crate::antfarm::{
    ant::AntsPlugin, background::BackgroundPlugin, elements::ElementsPlugin,
    gravity::GravityPlugin, root::RootPlugin,
};

use self::settings::{Probabilities, Settings};

#[derive(Resource)]
pub struct WorldState {
    // NOTE: These values should never be negative, but I am fearful of using `usize`
    width: isize,
    height: isize,
    surface_level: isize,
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

// TODO: maybe introduce a Tile concept?
#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    #[allow(dead_code)]
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    #[allow(dead_code)]
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const ONE: Self = Self::new(1, 1);
    #[allow(dead_code)]
    pub const NEG_ONE: Self = Self::new(-1, -1);

    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

impl Add for Position {
    type Output = Self;

    // TODO: Hexx uses const_add here?
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: I've declared const here to ensure they are accessed as resources

        // Defines the amount of time that should elapse between each physics step.
        const TIME_STEP: f32 = 1.0 / 60.0;

        const SETTINGS: Settings = Settings {
            compact_sand_depth: 15,
            initial_dirt_percent: 3.0 / 4.0,
            initial_ant_count: 20,
            ant_color: Color::rgb(0.584, 0.216, 0.859), // purple!
            probabilities: Probabilities {
                random_dig: 0.003,
                random_drop: 0.003,
                random_turn: 0.005,
                below_surface_dig: 0.10,
                above_surface_drop: 0.10,
            },
        };

        const WORLD_WIDTH: isize = 144;
        const WORLD_HEIGHT: isize = 81;
        const WORLD_STATE: WorldState = WorldState {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            // TODO: Double-check for off-by-one her
            surface_level: (WORLD_HEIGHT as f32
                - (WORLD_HEIGHT as f32 * SETTINGS.initial_dirt_percent))
                as isize,
        };

        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .insert_resource(SETTINGS)
        .insert_resource(WORLD_STATE)
        .insert_resource(WorldMap::new())
        .add_plugin(RootPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(ElementsPlugin)
        .add_plugin(AntsPlugin)
        .add_plugin(GravityPlugin);
    }
}
