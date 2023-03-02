mod air;
mod ant;
mod dirt;
mod gravity;
mod sand;
mod settings;

use crate::antfarm::{
    air::AirBundle,
    ant::{AntLabelBundle, AntSpriteBundle},
    dirt::DirtBundle,
    sand::SandBundle,
};
use bevy::{prelude::*, sprite::Anchor, time::FixedTimestep};

use self::{
    gravity::sand_gravity_system,
    settings::{Probabilities, Settings},
};

const WORLD_WIDTH: i32 = 144;
const WORLD_HEIGHT: i32 = 81;

// TODO: it kinda sucks having to declare this all the time?
#[derive(Component, PartialEq)]
pub enum Element {
    Air,
    Dirt,
    Sand,
}

#[derive(Component)]
pub struct Active(pub bool);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct WorldContainer;

// Defines the amount of time that should elapse between each physics step.
// NOTE: should probably run in 1/60 but slowing down for dev
const TIME_STEP: f32 = 10.0 / 60.0;

pub struct AntfarmPlugin;

impl Plugin for AntfarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                fit_canvas_to_parent: true,
                ..default()
            },
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

        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(window_resize_system)
                .with_system(sand_gravity_system),
        );
    }
}

fn window_resize_system(
    windows: Res<Windows>,
    mut query: Query<&mut Transform, With<WorldContainer>>,
) {
    let mut transform = query.single_mut();

    let (translation, scale) = get_world_container_transform(windows.get_primary().unwrap());

    transform.translation = translation;
    transform.scale = scale;
}

// World dimensions are integer values (144/81) but <canvas/> has variable, floating point dimensions.
// Determine a scaling factor so world fills available screen space.
fn get_world_container_transform(window: &Window) -> (Vec3, Vec3) {
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
    windows: Res<Windows>,
    settings: Res<Settings>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    // Wrap in container and shift to top-left viewport so 0,0 is top-left corner.
    let (translation, scale) = get_world_container_transform(&windows.get_primary().unwrap());
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
fn setup_background(parent: &mut ChildBuilder, settings: &Res<Settings>) {
    parent.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.529, 0.808, 0.922),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                get_surface_level(settings) as f32 + 1.0,
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });

    parent.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, -(get_surface_level(settings) as f32 + 1.0), 0.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.373, 0.290, 0.165),
            custom_size: Some(Vec2::new(
                WORLD_WIDTH as f32,
                WORLD_HEIGHT as f32 - (get_surface_level(settings) as f32 + 1.0),
            )),
            anchor: Anchor::TopLeft,
            ..default()
        },
        ..default()
    });
}

// Spawn interactive elements - air/dirt. Air isn't visible, background is revealed in its place.
fn setup_elements(parent: &mut ChildBuilder, settings: &Res<Settings>) {
    // Test Sand
    let sand_bundles = (0..1).flat_map(|row_index| {
        (0..WORLD_WIDTH).map(move |column_index| {
            // NOTE: row_index goes negative because 0,0 is top-left corner
            SandBundle::new(
                Vec3::new(column_index as f32, -row_index as f32, 1.0),
                Some(Vec2::ONE),
                true,
            )
        })
    });

    for sand_bundle in sand_bundles {
        parent.spawn(sand_bundle);
    }

    // Air & Dirt
    // NOTE: starting at 1 to skip sand
    let air_bundles = (1..get_surface_level(settings) + 1).flat_map(|row_index| {
        (0..WORLD_WIDTH).map(move |column_index| {
            // NOTE: row_index goes negative because 0,0 is top-left corner
            AirBundle::new(Vec3::new(column_index as f32, -row_index as f32, 1.0))
        })
    });

    for air_bundle in air_bundles {
        parent.spawn(air_bundle);
    }

    let dirt_bundles = ((get_surface_level(settings) + 1)..WORLD_HEIGHT).flat_map(|row_index| {
        (0..WORLD_WIDTH).map(move |column_index| {
            // NOTE: row_index goes negative because 0,0 is top-left corner
            DirtBundle::new(Vec3::new(column_index as f32, -row_index as f32, 1.0))
        })
    });

    for dirt_bundle in dirt_bundles {
        parent.spawn(dirt_bundle);
    }
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
                        parent.spawn(SandBundle::new(
                            Vec3::new(0.5, 0.33, 0.0),
                            Option::Some(Vec2::new(0.5, 0.5)),
                            false,
                        ));
                    }
                });
                parent.spawn(ant_bundle.2);
            });
    }
}

// TODO: Double-check for off-by-one here
fn get_surface_level(settings: &Res<Settings>) -> i32 {
    (WORLD_HEIGHT as f32 - (WORLD_HEIGHT as f32 * settings.initial_dirt_percent)) as i32
}
