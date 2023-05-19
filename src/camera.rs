use super::WorldMap;
use crate::pancam::{PanCam, PanCamPlugin};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Component)]
struct MainCamera;
pub struct CameraPlugin;

// Determine a scaling factor so world fills available screen space.
// NOTE: resize event is sent on load so this functions as an initializer, too.
fn window_resize(
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    world_map: Res<WorldMap>,
) {
    for resize_event in resize_events.iter() {
        let Ok(entity) = primary_window_query.get_single() else { continue };

        if resize_event.window == entity {
            let (mut transform, mut projection) = query.single_mut();

            let world_scale = (resize_event.width / *world_map.width() as f32)
                .max(resize_event.height / *world_map.height() as f32);

            transform.translation.x = resize_event.width / world_scale / 2.0;
            transform.translation.y = -resize_event.height / world_scale / 2.0;
            projection.scale = 1.0 / world_scale;
        }
    }
}

fn setup(mut commands: Commands, world_map: Res<WorldMap>) {
    commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .insert(PanCam {
            min_x: Some(0.0),
            min_y: Some(-world_map.height() as f32),
            max_x: Some(*world_map.width() as f32),
            max_y: Some(0.0),
            min_scale: 0.01,
            ..default()
        });
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PanCamPlugin::default());
        app.add_startup_system(setup).add_system(window_resize);
    }
}