mod air;
mod ant;
mod dirt;
mod point;
mod settings;
mod world;

use crate::antfarm::air::AirBundle;
use crate::antfarm::dirt::DirtBundle;
use bevy::{prelude::*, render::camera::RenderTarget};

// TODO: Don't hardcode this in the future. Need to support resizing to fit window, but starting simple.
const VIEWPORT_WIDTH: i32 = 400;
const VIEWPORT_HEIGHT: i32 = 200;

const WORLD_WIDTH: i32 = 144;
const WORLD_HEIGHT: i32 = 81;

const VIEWPORT_TOP_LEFT_POSITION: Vec3 = Vec3::new(
    VIEWPORT_WIDTH as f32 / -2.0,
    VIEWPORT_HEIGHT as f32 / 2.0,
    0.0,
);

// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

pub fn main(app: &mut App) {
    app.add_startup_system(setup);

    // app.add_system(my_cursor_system);

    // world::World::new(
    //     WORLD_WIDTH,
    //     WORLD_HEIGHT,
    //     settings::SETTINGS.initial_dirt_percent,
    //     settings::SETTINGS.initial_ant_count,
    // );
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Camera
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let world_container_bundle = SpatialBundle {
        transform: Transform {
            translation: VIEWPORT_TOP_LEFT_POSITION,
            scale: Vec3::new(20.0, 20.0, 0.0),
            ..default()
        },
        ..default()
    };

    commands
        .spawn(world_container_bundle)
        .with_children(|parent| {
            // Air Test
            let air_bundles =
                (0..20).map(|index| AirBundle::new(Vec3::new(index as f32, 0.0, 0.0)));

            for air_bundle in air_bundles {
                parent.spawn(air_bundle);
            }

            // Dirt Test
            let dirt_bundles =
                (0..20).map(|index| DirtBundle::new(Vec3::new(index as f32, -1.0, 0.0)));

            for dirt_bundle in dirt_bundles {
                parent.spawn(dirt_bundle);
            }
        });
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
