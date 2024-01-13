use crate::{app_state::AppState, story::grid::Grid};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

use self::pancam::{PanCam, PanCamPlugin};

use super::common::ui::VisibleGrid;

mod pancam;

#[derive(Component)]
pub struct MainCamera;

/// Calculate the scale which will minimally cover the window with a grid.
fn get_best_fit_scale(
    window_width: f32,
    window_height: f32,
    grid_width: f32,
    grid_height: f32,
) -> f32 {
    1.0 / (window_width / grid_width).max(window_height / grid_height)
}

/// Keep in mind that window_resize fires on load due to `fit_canvas_to_parent: true` resizing the <canvas />
fn window_resize(
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
    mut resize_events: EventReader<WindowResized>,
    mut main_camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let visible_grid = match grid_query.get(visible_grid_entity) {
        Ok(visible_grid) => visible_grid,
        Err(_) => return,
    };

    let primary_window_entity = primary_window_query.single();

    for resize_event in resize_events.read() {
        if resize_event.window == primary_window_entity {
            main_camera_query.single_mut().scale = get_best_fit_scale(
                resize_event.width,
                resize_event.height,
                visible_grid.width() as f32,
                visible_grid.height() as f32,
            );
        }
    }
}

/// Keep in mind that window.width() doesn't fit the viewport until `fit_canvas_to_parent: true` resizes the <canvas />
fn scale_projection(
    mut main_camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid>,
) {
    if !visible_grid.is_changed() {
        return;
    }

    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let visible_grid = match grid_query.get(visible_grid_entity) {
        Ok(visible_grid) => visible_grid,
        Err(_) => return,
    };

    // let visible_grid = visible_grid_query.single();
    let primary_window = primary_window_query.single();

    main_camera_query.single_mut().scale = get_best_fit_scale(
        primary_window.width(),
        primary_window.height(),
        visible_grid.width() as f32,
        visible_grid.height() as f32,
    );
}

fn insert_pancam(
    main_camera_query: Query<Entity, With<MainCamera>>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid>,
    mut commands: Commands,
) {
    if !visible_grid.is_changed() {
        return;
    }

    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let visible_grid = match grid_query.get(visible_grid_entity) {
        Ok(visible_grid) => visible_grid,
        Err(_) => return,
    };

    commands.entity(main_camera_query.single()).insert(PanCam {
        min_x: Some(-visible_grid.width() as f32 / 2.0),
        min_y: Some(-visible_grid.height() as f32 / 2.0),
        max_x: Some(visible_grid.width() as f32 / 2.0),
        max_y: Some(visible_grid.height() as f32 / 2.0),
        min_scale: 0.01,
        ..default()
    });
}

pub fn setup(mut commands: Commands) {
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

        app.add_systems(OnEnter(AppState::BeginSetup), setup);
        app.add_systems(OnEnter(AppState::Cleanup), teardown);

        app.add_systems(
            Update,
            window_resize.run_if(resource_exists::<VisibleGrid>()),
        );
        app.add_systems(
            Update,
            (insert_pancam, scale_projection).run_if(resource_exists::<VisibleGrid>()),
        );
    }
}
