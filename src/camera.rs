use crate::{
    pancam::{PanCam, PanCamPlugin},
    settings::{pre_setup_settings, Settings},
    story_state::{restart_story, StoryState},
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
    mut main_camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    settings: Res<Settings>,
) {
    for resize_event in resize_events.iter() {
        let Ok(entity) = primary_window_query.get_single() else {
            continue;
        };

        if resize_event.window == entity {
            let max_ratio = (resize_event.width / settings.world_width as f32)
                .max(resize_event.height / settings.world_height as f32);

            main_camera_query.single_mut().scale = 1.0 / max_ratio;
        }
    }
}

// TODO: Instead of scaling to fit the world I want to zoom and focus on queen ant, but still enforce same max/min bounds.
fn scale_to_world(
    mut commands: Commands,
    settings: Res<Settings>,
    mut main_camera_query: Query<(Entity, &mut OrthographicProjection), With<MainCamera>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_query.single();

    // NOTE: This calculation is "wrong" on first load because resize has yet to occur.
    // This calculation is correct when resetting world state post-load, though.
    let max_ratio = (primary_window.width() / settings.world_width as f32)
        .max(primary_window.height() / settings.world_height as f32);

    let (main_camera_entity, mut projection) = main_camera_query.single_mut();

    projection.scale = 1.0 / max_ratio;

    commands.entity(main_camera_entity).insert(PanCam {
        min_x: Some(-settings.world_width as f32 / 2.0),
        min_y: Some(-settings.world_height as f32 / 2.0),
        max_x: Some(settings.world_width as f32 / 2.0),
        max_y: Some(settings.world_height as f32 / 2.0),
        min_scale: 0.01,
        ..default()
    });
}

// TODO: Make this more loosely coupled so I can just tell camera to focus a specific position.
/// Find the queen ant, center camera position over queen ant, do so at a scale that is visually pleasing for focusing on the queen.
// fn focus_queen(
//     ant_query: Query<(&Position, &AntRole)>,
//     mut events: EventWriter<Pan>,
//     world_map: Res<WorldMap>,
// ) {
//     let queen_ant_position = ant_query
//         .iter()
//         .find(|(_, role)| **role == AntRole::Queen)
//         .map(|(position, _)| position);

//     if let Some(queen_ant_position) = queen_ant_position {
//         let queen_world_position = queen_ant_position.as_world_position(&world_map);

//         events.send(Pan(queen_world_position.truncate()));
//     }
// }

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

        // TODO: This is still wrong because waiting for predefined settings isn't useful if loaded settings are different.
        app.add_systems(
            OnEnter(StoryState::Initializing),
            (
                setup,
                apply_deferred,
                scale_to_world.after(pre_setup_settings),
            )
                .chain(),
        );

        // app.add_systems(
        //     OnEnter(StoryState::Telling),
        //     (focus_queen).chain(),
        // );

        app.add_systems(OnEnter(StoryState::Cleanup), teardown.before(restart_story));
        app.add_systems(Update, window_resize.run_if(resource_exists::<Settings>()));
    }
}
