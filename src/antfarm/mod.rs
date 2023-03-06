mod ant;
mod background;
mod elements;
mod gravity;
mod settings;

use std::fmt;

use bevy::{prelude::*, window::PrimaryWindow};

use self::{
    ant::setup_ants,
    background::setup_background,
    elements::setup_elements,
    gravity::sand_gravity_system,
    settings::{Probabilities, Settings},
};

const WORLD_WIDTH: usize = 144;
const WORLD_HEIGHT: usize = 81;

// TODO: it kinda sucks having to declare this all the time?
#[derive(Component, PartialEq, Copy, Clone, Debug)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
pub struct WorldContainer;

// Defines the amount of time that should elapse between each physics step.
// NOTE: should probably run in 1/60 but slowing down for dev
const TIME_STEP: f32 = 10.0 / 60.0;

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }));

        app.insert_resource(Settings {
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
        });

        app.add_startup_system(setup);

        app.add_systems(
            (window_resize_system, sand_gravity_system).in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP));
    }
}

fn window_resize_system(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<WorldContainer>>,
) {
    let Ok(primary_window) = primary_window_query.get_single() else {
        return;
    };

    let mut transform = query.single_mut();

    let (translation, scale) = get_world_container_transform(primary_window);

    transform.translation = translation;
    transform.scale = scale;
}

// World dimensions are integer values (144/81) but <canvas/> has variable, floating point dimensions.
// Determine a scaling factor so world fills available screen space.
pub fn get_world_container_transform(window: &Window) -> (Vec3, Vec3) {
    let world_scale =
        (window.width() / WORLD_WIDTH as f32).max(window.height() / WORLD_HEIGHT as f32);

    info!(
        "Window Height/Width: {}/{}, World Scale: {}",
        window.width(),
        window.height(),
        world_scale,
    );

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
) {
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let Ok(primary_window) = primary_window_query.get_single() else {
        return;
    };

    commands.spawn((Camera2dBundle::default(), MainCamera));

    // TODO: this feels super wrong
    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let (translation, scale) = get_world_container_transform(primary_window);
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
            setup_background(parent, &settings);
            setup_elements(parent, &settings);
            setup_ants(parent, &asset_server, &settings);
        });
}

// TODO: Double-check for off-by-one here
pub fn get_surface_level(initial_dirt_percent: f32) -> usize {
    (WORLD_HEIGHT as f32 - (WORLD_HEIGHT as f32 * initial_dirt_percent)) as usize
}
