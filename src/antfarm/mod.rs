mod ant;
mod background;
mod elements;
mod gravity;
mod settings;
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use std::ops::Add;

use self::{
    ant::setup_ants,
    background::setup_background,
    elements::setup_elements,
    gravity::sand_gravity_system,
    settings::{Probabilities, Settings},
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct WorldContainer;

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
    pub fn new() -> Self {
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
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const ONE: Self = Self::new(1, 1);
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
        // NOTE: should probably run in 1/60 but slowing down for dev
        const TIME_STEP: f32 = 10.0 / 60.0;

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
        .add_startup_system(setup)
        .add_systems(
            (window_resize_system, sand_gravity_system).in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

fn window_resize_system(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<WorldContainer>>,
    world_state: Res<WorldState>,
) {
    let Ok(primary_window) = primary_window_query.get_single() else {
        return;
    };

    let mut transform = query.single_mut();

    let (translation, scale) = get_world_container_transform(primary_window, &world_state);

    transform.translation = translation;
    transform.scale = scale;
}

// World dimensions are integer values (144/81) but <canvas/> has variable, floating point dimensions.
// Determine a scaling factor so world fills available screen space.
fn get_world_container_transform(window: &Window, world_state: &Res<WorldState>) -> (Vec3, Vec3) {
    let world_scale = (window.width() / world_state.width as f32)
        .max(window.height() / world_state.height as f32);

    // info!(
    //     "Window Height/Width: {}/{}, World Scale: {}",
    //     window.width(),
    //     window.height(),
    //     world_scale,
    // );

    (
        // translation:
        Vec3::new(window.width() / -2.0, window.height() / 2.0, 0.0),
        // scale:
        Vec3::new(world_scale, world_scale, 1.0),
    )
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<Settings>,
    world_state: Res<WorldState>,
    world_map: ResMut<WorldMap>,
) {
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let Ok(primary_window) = primary_window_query.get_single() else {
        return;
    };

    commands.spawn((Camera2dBundle::default(), MainCamera));

    // TODO: this feels super wrong
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let (translation, scale) = get_world_container_transform(primary_window, &world_state);
    let world_container_bundle = SpatialBundle {
        transform: Transform {
            translation,
            scale,
            ..default()
        },
        ..default()
    };

    commands
        .spawn((world_container_bundle, WorldContainer))
        .with_children(|parent| {
            // TODO: This sucks! One of the main perks of Resource is to use DI to gain access rather than passing through hierarchy chain
            setup_background(parent, &world_state);
            setup_elements(parent, &world_state, world_map);
            setup_ants(parent, &asset_server, &settings);
        });
}
