use crate::{
    world_map::WorldMap,
    pancam::{PanCam, PanCamPlugin},
    story_state::{on_story_cleanup, StoryState},
};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Component)]
pub struct MainCamera;

// Determine a scaling factor so world fills available screen space.
// NOTE: resize event is sent on load (due to fit_canvas_to_parent: true)
fn window_resize(
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    world_map: Res<WorldMap>,
) {
    for resize_event in resize_events.iter() {
        let Ok(entity) = primary_window_query.get_single() else {
            continue;
        };

        if resize_event.window == entity {
            let max_ratio = (resize_event.width / *world_map.width() as f32)
                .max(resize_event.height / *world_map.height() as f32);

            query.single_mut().scale = 1.0 / max_ratio;
        }
    }
}

fn scale_to_world(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    mut main_camera_query: Query<(Entity, &mut OrthographicProjection), With<MainCamera>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_query.single();

    // NOTE: This calculation is "wrong" on first load because resize has yet to occur.
    // This calculation is correct when resetting world state post-load, though.
    let max_ratio = (primary_window.width() / *world_map.width() as f32)
        .max(primary_window.height() / *world_map.height() as f32);

    let (main_camera_entity, mut projection) = main_camera_query.single_mut();

    projection.scale = 1.0 / max_ratio;

    commands.entity(main_camera_entity).insert(PanCam {
        min_x: Some(-world_map.width() as f32 / 2.0),
        min_y: Some(-world_map.height() as f32 / 2.0),
        max_x: Some(*world_map.width() as f32 / 2.0),
        max_y: Some(*world_map.height() as f32 / 2.0),
        min_scale: 0.01,
        ..default()
    });
}

pub fn setup(mut commands: Commands) {
    // This needs to run early so main menu is visible even without a WorldMap created.
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub fn teardown(mut commands: Commands, query: Query<Entity, With<MainCamera>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin::default());

        app.add_systems(OnEnter(StoryState::Initializing), setup);

        app.add_systems(OnEnter(StoryState::Telling), scale_to_world);

        app.add_systems(
            OnEnter(StoryState::Cleanup),
            teardown.before(on_story_cleanup),
        );

        // TODO: This should probably include "Over" until I split states out
        app.add_systems(Update, window_resize.run_if(in_state(StoryState::Telling)));
    }
}
