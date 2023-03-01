mod air;
mod ant;
mod dirt;
mod point;
mod sand;
mod settings;
mod world;
use rand::Rng;

use crate::antfarm::air::AirBundle;
use crate::antfarm::ant::AntLabelBundle;
use crate::antfarm::ant::AntSpriteBundle;
use crate::antfarm::dirt::DirtBundle;
use crate::antfarm::sand::SandBundle;
use bevy::prelude::*;
use bevy::time::FixedTimestep;

const WORLD_WIDTH: i32 = 144;
const WORLD_HEIGHT: i32 = 81;

// TODO: Should this be -1?
const SURFACE_LEVEL: i32 =
    (WORLD_HEIGHT as f32 - (WORLD_HEIGHT as f32 * settings::SETTINGS.initial_dirt_percent)) as i32;

// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct WorldContainer;

#[derive(Resource)]
struct UiFont(Handle<Font>);

// TODO: Should this apply to window resize?
// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// TODO: probably want a resource for settings

pub fn main(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            fit_canvas_to_parent: true,
            canvas: Option::Some("#canvas".to_string()),
            ..default()
        },
        ..default()
    }));

    app.add_startup_system(setup);

    app.add_system_set(
        SystemSet::new()
            .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
            .with_system(window_resize_system),
    );

    // app.add_system(my_cursor_system);
}

fn window_resize_system(
    windows: Res<Windows>,
    mut query: Query<(&WorldContainer, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    web_sys::console::log_3(
        &"Window Size".into(),
        &window.width().to_string().into(),
        &window.height().to_string().into(),
    );

    let world_container_transform = get_world_container_transform(window);

    for (_, mut transform) in query.iter_mut() {
        transform.translation = world_container_transform.0;
        transform.scale = world_container_transform.1
    }
}

// World dimensions are integer values (144/81) but <canvas/> has variable, floating point dimensions.
// Determine a scaling factor so world fills available screen space.
fn get_world_container_transform(window: &Window) -> (Vec3, Vec3) {
    let world_scale =
        (window.width() / WORLD_WIDTH as f32).max(window.height() / WORLD_HEIGHT as f32);

    web_sys::console::log_4(
        &"Window Size / World Scale".into(),
        &window.width().to_string().into(),
        &window.height().to_string().into(),
        &world_scale.to_string().into(),
    );

    (
        // translation:
        Vec3::new(window.width() / -2.0, window.height() / 2.0, 0.0),
        // scale:
        Vec3::new(world_scale, world_scale, 0.0),
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
) {
    web_sys::console::log_2(&"Surface Level".into(), &SURFACE_LEVEL.to_string().into());

    // commands.insert_resource(world::World::new(
    //     WORLD_WIDTH,
    //     WORLD_HEIGHT,
    //     settings::SETTINGS.initial_dirt_percent,
    //     settings::SETTINGS.initial_ant_count,
    // ));

    let world_container_transform = get_world_container_transform(&windows.get_primary().unwrap());

    // Camera
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let world_container_bundle = SpatialBundle {
        transform: Transform {
            translation: world_container_transform.0,
            scale: world_container_transform.1,
            ..default()
        },
        ..default()
    };

    commands
        .spawn((world_container_bundle, WorldContainer))
        .with_children(|parent| {
            setup_elements(parent);
            setup_ants(parent, &asset_server);
        });
}

// Spawn elements - air, dirt, tunnel, sand that are initially present in the simulation.
// TODO: need to support tunnels and sand, but original code didn't explicitly have air/tunnel elements, just colored the background
fn setup_elements(parent: &mut ChildBuilder) {
    // Air & Dirt
    let air_bundles = (0..SURFACE_LEVEL + 1).flat_map(|row_index| {
        (0..WORLD_WIDTH).map(move |column_index| {
            // NOTE: row_index goes negative because 0,0 is top-left corner
            AirBundle::new(Vec3::new(column_index as f32, -row_index as f32, 0.0))
        })
    });

    for air_bundle in air_bundles {
        parent.spawn(air_bundle);
    }

    let dirt_bundles = ((SURFACE_LEVEL + 1)..WORLD_HEIGHT).flat_map(|row_index| {
        (0..WORLD_WIDTH).map(move |column_index| {
            // NOTE: row_index goes negative because 0,0 is top-left corner
            DirtBundle::new(Vec3::new(column_index as f32, -row_index as f32, 0.0))
        })
    });

    for dirt_bundle in dirt_bundles {
        parent.spawn(dirt_bundle);
    }
}

fn setup_ants(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
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
    //             settings::SETTINGS.ant_color,
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
                settings::SETTINGS.ant_color,
                ant::Facing::Left,
                ant::Angle::Zero,
                ant::Behavior::Carrying,
                &asset_server,
            ),
            AntLabelBundle::new("ant1".to_string(), &asset_server),
        ),
        (
            Vec3::new(10.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Left,
                ant::Angle::Ninety,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant2".to_string(), &asset_server),
        ),
        (
            Vec3::new(15.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Left,
                ant::Angle::OneHundredEighty,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant3".to_string(), &asset_server),
        ),
        (
            Vec3::new(20.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Left,
                ant::Angle::TwoHundredSeventy,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant4".to_string(), &asset_server),
        ),
        (
            Vec3::new(25.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Right,
                ant::Angle::Zero,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant5".to_string(), &asset_server),
        ),
        (
            Vec3::new(30.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Right,
                ant::Angle::Ninety,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant6".to_string(), &asset_server),
        ),
        (
            Vec3::new(35.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Right,
                ant::Angle::OneHundredEighty,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant7".to_string(), &asset_server),
        ),
        (
            Vec3::new(40.0, -5.0, 1.0),
            AntSpriteBundle::new(
                settings::SETTINGS.ant_color,
                ant::Facing::Right,
                ant::Angle::TwoHundredSeventy,
                ant::Behavior::Wandering,
                &asset_server,
            ),
            AntLabelBundle::new("ant8".to_string(), &asset_server),
        ),
    ];

    for ant_bundle in test_ant_bundles {
        let ant_label_container_bundle = SpatialBundle {
            transform: Transform {
                translation: ant_bundle.0,
                ..default()
            },
            ..default()
        };

        let angle_degrees = match ant_bundle.1.angle {
            ant::Angle::Zero => 0,
            ant::Angle::Ninety => 90,
            ant::Angle::OneHundredEighty => 180,
            ant::Angle::TwoHundredSeventy => 270,
        };
        // TODO: is this a bad architectural decision? technically I am thinking about mirroring improperly by inverting angle when x is flipped?
        let x_flip = if ant_bundle.1.facing == ant::Facing::Left {
            -1.0
        } else {
            1.0
        };

        let angle_radians = angle_degrees as f32 * std::f32::consts::PI / 180.0 * x_flip;
        let rotation = Quat::from_rotation_z(angle_radians);

        // TODO: Maybe I am thinking of this wrong? Instead of giving both sand and ant to a contrived ant container
        // perhaps I should be giving sand to the ant container?
        let ant_container_bundle = SpatialBundle {
            transform: Transform {
                rotation,
                scale: Vec3::new(x_flip, 1.0, 1.0),
                translation: Vec3::new(0.5, -0.5, 100.0),
                ..default()
            },
            ..default()
        };

        let is_carrying = ant_bundle.1.behavior == ant::Behavior::Carrying;

        parent
            // Wrap label and ant with common parent to associate their movement.
            .spawn(ant_label_container_bundle)
            .with_children(|parent| {
                // Wrap ant sprite and optional sand with common parent to associate their rotation and scale.
                parent.spawn(ant_container_bundle).with_children(|parent| {
                    parent.spawn(ant_bundle.1);

                    if is_carrying {
                        parent.spawn(SandBundle::new(
                            Vec3::new(0.5, 0.33, 0.0),
                            Option::Some(Vec2::new(0.5, 0.5)),
                        ));
                    }
                });
                // TODO: This label disappears sometimes and doesn't seem to be related to zindex?
                parent.spawn(ant_bundle.2);
            });
    }
}

// fn my_cursor_system(
//     // need to get window dimensions
//     windows: Res<Windows>,
//     // query to get camera transform
//     camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
// ) {
//     // get the camera info and transform
//     // assuming there is exactly one main camera entity, so query::single() is OK
//     let (camera, camera_transform) = camera_q.single();

//     // get the window that the camera is displaying to (or the primary window)
//     let window = if let RenderTarget::Window(id) = camera.target {
//         windows.get(id).unwrap()
//     } else {
//         windows.get_primary().unwrap()
//     };

//     // check if the cursor is inside the window and get its position
//     // then, ask bevy to convert into world coordinates, and truncate to discard Z
//     if let Some(world_position) = window
//         .cursor_position()
//         .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
//         .map(|ray| ray.origin.truncate())
//     {
//         web_sys::console::log_3(
//             &"Viewport coords: {}/{}".into(),
//             &window
//                 .cursor_position()
//                 .ok_or("no item")
//                 .unwrap()
//                 .x
//                 .to_string()
//                 .into(),
//             &window
//                 .cursor_position()
//                 .ok_or("no item")
//                 .unwrap()
//                 .y
//                 .to_string()
//                 .into(),
//         );

//         web_sys::console::log_3(
//             &"World coords: {}/{}".into(),
//             &world_position.x.to_string().into(),
//             &world_position.y.to_string().into(),
//         );
//     }
// }
