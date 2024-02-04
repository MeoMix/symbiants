pub mod grid;
pub mod position;

use self::position::Position;
use super::{
    app_state::{
        begin_story, continue_startup, finalize_startup, post_setup_clear_change_detection,
        restart, AppState,
    },
    external_event::{initialize_external_event_resources, remove_external_event_resources},
    save::{
        bind_save_onbeforeunload, delete_save_file, initialize_save_resources, load,
        remove_save_resources, save, unbind_save_onbeforeunload,
    },
    settings::{initialize_settings_resources, register_settings, remove_settings_resources},
    story_time::{
        initialize_story_time_resources, register_story_time, remove_story_time_resources,
        setup_story_time, update_story_real_world_time, update_time_scale, StoryPlaybackState,
    },
    CleanupSet, FinishSetupSet,
};
use bevy::prelude::*;

// This maps to AtNest or AtCrater
/// Use an empty trait to mark Nest and Crater zones to ensure strong type safety in generic systems.
pub trait Zone: Component {}

pub fn register_common(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Entity>();
    app_type_registry.write().register::<Option<Entity>>();
    app_type_registry.write().register::<Position>();
}

pub fn despawn_model<Model: Component>(
    model_query: Query<Entity, With<Model>>,
    mut commands: Commands,
) {
    for model_entity in model_query.iter() {
        commands.entity(model_entity).despawn();
    }
}

pub struct CommonSimulationPlugin;

impl Plugin for CommonSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::BeginSetup),
            (register_settings, register_common, register_story_time),
        );

        app.add_systems(
            OnEnter(AppState::TryLoadSave),
            (
                (initialize_save_resources, apply_deferred).chain(),
                load.pipe(continue_startup),
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (
                (initialize_settings_resources, apply_deferred).chain(),
                finalize_startup,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (initialize_story_time_resources, apply_deferred).chain(),
                (initialize_external_event_resources, apply_deferred).chain(),
                // TODO: Feels weird to say saving is part of the simulation logic.
                bind_save_onbeforeunload,
                post_setup_clear_change_detection,
            )
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        // IMPORTANT: setup_story_time sets FixedTime.accumulated which is reset when transitioning between schedules.
        // If this is ran OnEnter FinishSetup then the accumulated time will be reset to zero before FixedUpdate runs.
        app.add_systems(OnExit(AppState::FinishSetup), setup_story_time);

        app.add_systems(
            OnEnter(AppState::PostSetupClearChangeDetection),
            begin_story,
        );

        app.add_systems(
            Update,
            update_time_scale.run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            Update,
            update_story_real_world_time.run_if(in_state(AppState::TellStory)),
        );

        // Saving in WASM writes to local storage which requires dedicated support.
        app.add_systems(
            PostUpdate,
            // Saving is an expensive operation. Skip while fast-forwarding for performance.
            // TODO: It's weird (incorrect) that this is declared in `simulation` but that the `save` directory is external to simulation.
            // I think this should get moved up a level.
            save.run_if(
                in_state(AppState::TellStory).and_then(in_state(StoryPlaybackState::Playing)),
            ),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                unbind_save_onbeforeunload,
                delete_save_file,
                remove_story_time_resources,
                remove_settings_resources,
                remove_save_resources,
                remove_external_event_resources,
                restart,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
