mod ant;
mod elements;
mod gravity;
mod settings;

use std::fmt;

use crate::antfarm::ant::{AntLabelBundle, AntSpriteBundle};
use bevy::{prelude::*, sprite::Anchor, window::PrimaryWindow};

use self::{
    elements::{setup_elements, ElementBundle},
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

// Spawn non-interactive background (sky blue / tunnel brown)
pub fn setup_background(parent: &mut ChildBuilder, settings: &Res<Settings>) {
    parent.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                get_surface_level(settings.initial_dirt_percent) as f32 + 1.0,
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });

    parent.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(
                0.0,
                -(get_surface_level(settings.initial_dirt_percent) as f32 + 1.0),
                0.0,
            ),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                WORLD_HEIGHT as f32
                    - (get_surface_level(settings.initial_dirt_percent) as f32 + 1.0),
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });
}

fn setup_ants(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    settings: &Res<Settings>,
) {
    // let ant_bundles = (0..8).map(|_| {
    //     // Put the ant at a random location along the x-axis that fits within the bounds of the world.
    //     // TODO: technically old code was .round() and now it's just floored implicitly
    //     let x = rand::thread_rng().gen_range(0..1000) as f32 % WORLD_WIDTH as f32;
    //     // Put the ant on the dirt.
    //     let y = SURFACE_LEVEL as f32;

    //     // Randomly position ant facing left or right.
    //     let facing = if rand::thread_rng().gen_range(0..10) < 5 {
    //         ant::Facing::Left
    //     } else {
    //         ant::Facing::Right
    //     };

    //     (
    //         Vec3::new(x, -y, 0.0),
    //         AntSpriteBundle::new(
    //             settings.ant_color,
    //             facing,
    //             ant::Angle::Zero,
    //             ant::Behavior::Wandering,
    //             &asset_server,
    //         ),
    //         AntLabelBundle::new("Test Name".to_string(), &asset_server),
    //     )
    // });

    let test_ant_bundles = [
        (
            Vec3::new(5.0, -5.0, 100.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Left,
                ant::AntAngle::Zero,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant1".to_string(), &asset_server),
        ),
        (
            Vec3::new(10.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Left,
                ant::AntAngle::Ninety,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant2".to_string(), &asset_server),
        ),
        (
            Vec3::new(15.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Left,
                ant::AntAngle::OneHundredEighty,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant3".to_string(), &asset_server),
        ),
        (
            Vec3::new(20.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Left,
                ant::AntAngle::TwoHundredSeventy,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant4".to_string(), &asset_server),
        ),
        (
            Vec3::new(25.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Right,
                ant::AntAngle::Zero,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant5".to_string(), &asset_server),
        ),
        (
            Vec3::new(30.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Right,
                ant::AntAngle::Ninety,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant6".to_string(), &asset_server),
        ),
        (
            Vec3::new(35.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Right,
                ant::AntAngle::OneHundredEighty,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant7".to_string(), &asset_server),
        ),
        (
            Vec3::new(40.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings.ant_color,
                ant::AntFacing::Right,
                ant::AntAngle::TwoHundredSeventy,
                ant::AntBehavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant8".to_string(), &asset_server),
        ),
    ];

    for ant_bundle in test_ant_bundles {
        let is_carrying = ant_bundle.1.behavior == ant::AntBehavior::Carrying;

        parent
            // Wrap label and ant with common parent to associate their movement, but not their rotation.
            .spawn((
                SpatialBundle {
                    transform: Transform {
                        translation: ant_bundle.0,
                        ..default()
                    },
                    ..default()
                },
                ant::Ant,
            ))
            .with_children(|parent| {
                // Make sand a child of ant so they share rotation.
                parent.spawn(ant_bundle.1).with_children(|parent| {
                    if is_carrying {
                        // NOTE: sand carried by ants is not "affected by gravity" intentionally
                        // There might need to be a better way of handling this once ant gravity is implemented
                        parent.spawn(ElementBundle::create_sand(
                            Vec3::new(0.5, 0.33, 0.0),
                            Option::Some(Vec2::new(0.5, 0.5)),
                        ));
                    }
                });
                parent.spawn(ant_bundle.2);
            });
    }
}

// TODO: Double-check for off-by-one here
pub fn get_surface_level(initial_dirt_percent: f32) -> usize {
    (WORLD_HEIGHT as f32 - (WORLD_HEIGHT as f32 * initial_dirt_percent)) as usize
}
