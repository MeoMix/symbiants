use crate::{
    grid::{setup_world_map, WorldMap},
    pancam::{PanCam, PanCamPlugin},
    story_state::StoryState,
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

fn setup(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_query.single();

    // NOTE: This calculation is "wrong" on first load because resize has yet to occur.
    // This calculation is correct when resetting world state post-load, though.
    let max_ratio = (primary_window.width() / *world_map.width() as f32)
        .max(primary_window.height() / *world_map.height() as f32);

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 1.0 / max_ratio;

    commands.spawn((camera_bundle, MainCamera)).insert(PanCam {
        min_x: Some(-world_map.width() as f32 / 2.0),
        min_y: Some(-world_map.height() as f32 / 2.0),
        max_x: Some(*world_map.width() as f32 / 2.0),
        max_y: Some(*world_map.height() as f32 / 2.0),
        min_scale: 0.01,
        ..default()
    });
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

        app.add_systems(
            OnEnter(StoryState::NotStarted),
            setup.after(setup_world_map),
        );

        // TODO: This isn't right because it'll destroy the camera on gameover but that isn't what is wanted
        app.add_systems(OnExit(StoryState::Telling), teardown);

        app.add_systems(Update, window_resize);
    }
}
