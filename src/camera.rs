use crate::{
    grid::{setup_world_map, WorldMap},
    pancam::{PanCam, PanCamPlugin},
};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

#[derive(Component)]
pub struct MainCamera;

// Determine a scaling factor so world fills available screen space.
// NOTE: resize event is sent on load (due to fit_canvas_to_parent: true) so this functions as an initializer, too.
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

fn setup(mut commands: Commands, world_map: Res<WorldMap>) {
    commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .insert(PanCam {
            min_x: Some(-world_map.width() as f32 / 2.0),
            min_y: Some(-world_map.height() as f32 / 2.0),
            max_x: Some(*world_map.width() as f32 / 2.0),
            max_y: Some(*world_map.height() as f32 / 2.0),
            min_scale: 0.01,
            ..default()
        });
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin::default());
        app.add_systems(Startup, setup.after(setup_world_map));
        app.add_systems(Update, window_resize);
    }
}
