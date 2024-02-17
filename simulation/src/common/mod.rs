pub mod ant;
pub mod element;
pub mod grid;
pub mod pheromone;
pub mod position;

use crate::{
    app_state::check_story_over, crater_simulation::crater::AtCrater,
    nest_simulation::nest::AtNest, story_time::set_rate_of_time,
};

use self::{
    ant::{
        death::on_ants_add_dead,
        digestion::ants_digestion,
        hunger::{ants_hunger_act, ants_hunger_regurgitate, ants_hunger_tick},
        initiative::ants_initiative,
        register_ant, AntAteFoodEvent,
    },
    element::register_element,
    pheromone::register_pheromone,
    position::Position,
};
use super::{
    app_state::{
        begin_story, finalize_startup, post_setup_clear_change_detection,
        restart, AppState,
    },
    common::element::map_element_to_marker,
    external_event::{
        initialize_external_event_resources, process_external_event,
        remove_external_event_resources,
    },
    save::{
        bind_save_onbeforeunload, delete_save_file, initialize_save_resources, load_save_file,
        remove_save_resources, save, unbind_save_onbeforeunload,
    },
    settings::{initialize_settings_resources, register_settings, remove_settings_resources},
    story_time::{
        initialize_story_time_resources, register_story_time, remove_story_time_resources,
        setup_story_time, update_story_elapsed_ticks, update_story_real_world_time,
        update_time_scale, StoryPlaybackState,
    },
    CleanupSet, FinishSetupSet, SimulationTickSet, SimulationUpdate,
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

pub fn despawn_model<Model: Component, Z: Zone>(
    model_query: Query<Entity, (With<Model>, With<Z>)>,
    mut commands: Commands,
) {
    for model_entity in model_query.iter() {
        commands.entity(model_entity).despawn();
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
pub enum LoadProgress {
    #[default]
    NotStarted,
    Loading,
    Success,
    Failure,
}

#[derive(Resource, Default, Debug)]
pub struct SimulationLoadProgress {
    pub save_file: LoadProgress,
}

pub fn initialize_loading_resources(mut commands: Commands) {
    commands.init_resource::<SimulationLoadProgress>();
}

pub fn remove_loading_resources(mut commands: Commands) {
    commands.remove_resource::<SimulationLoadProgress>();
}

pub struct CommonSimulationPlugin;

impl Plugin for CommonSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AntAteFoodEvent>();

        app.add_systems(
            Startup,
            (
                register_settings,
                register_common,
                register_story_time,
                register_element,
                register_pheromone,
                register_ant,
            ),
        );

        app.add_systems(
            OnEnter(AppState::Loading),
            (
                initialize_loading_resources,
                initialize_save_resources,
                apply_deferred,
                load_save_file,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (initialize_settings_resources, finalize_startup).chain(),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (
                    initialize_story_time_resources,
                    apply_deferred,
                    setup_story_time,
                    set_rate_of_time,
                )
                    .chain(),
                initialize_external_event_resources,
                bind_save_onbeforeunload,
                // TODO: This needs to run once before Simulation runs because UI update runs before first simulation tick.
                // If this doesn't run, UI filter queries like Without<Air> won't properly exclude.
                map_element_to_marker,
                post_setup_clear_change_detection,
            )
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            OnEnter(AppState::PostSetupClearChangeDetection),
            begin_story,
        );

        app.add_systems(
            SimulationUpdate,
            (
                process_external_event::<AtNest>,
                process_external_event::<AtCrater>,
                apply_deferred,
                map_element_to_marker,
                apply_deferred,
            )
                .chain()
                .in_set(SimulationTickSet::First),
        );

        app.add_systems(
            SimulationUpdate,
            (
                apply_deferred,
                (
                    ants_digestion::<AtNest>,
                    ants_digestion::<AtCrater>,
                    ants_hunger_tick::<AtNest>,
                    ants_hunger_tick::<AtCrater>,
                    ants_hunger_act::<AtNest>,
                    ants_hunger_act::<AtCrater>,
                    apply_deferred,
                    ants_hunger_regurgitate::<AtNest>,
                    ants_hunger_regurgitate::<AtCrater>,
                    apply_deferred,
                )
                    .chain(),
                on_ants_add_dead::<AtNest>,
                on_ants_add_dead::<AtCrater>,
            )
                .chain()
                .run_if(not(in_state(StoryPlaybackState::Paused)))
                .in_set(SimulationTickSet::SimulationTick),
        );

        app.add_systems(
            SimulationUpdate,
            (
                // TODO: maybe want to run initative at the end of the simulation tick but not in PostSimulationTick? :s
                apply_deferred,
                // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                ants_initiative::<AtNest>,
                ants_initiative::<AtCrater>,
                update_story_elapsed_ticks,
            )
                .chain()
                .in_set(SimulationTickSet::PostSimulationTick)
                .run_if(not(in_state(StoryPlaybackState::Paused))),
        );

        // TODO: Maybe (some?) of these should just run in Update?
        // Ending story seems like it should check every tick, but updating element exposure/updating story time seems OK to run just in Update?
        app.add_systems(
            SimulationUpdate,
            (
                // If this doesn't run then when user spawns elements they won't gain exposure if simulation is paused.
                apply_deferred,
                // TODO: Need to run this after simulation (as well as before) to ensure UI layer reads fresh data.
                map_element_to_marker,
                apply_deferred,
                check_story_over,
                // rate_of_time needs to run when app is paused because fixed_time accumulations need to be cleared while app is paused
                // to prevent running FixedUpdate schedule repeatedly (while no-oping) when coming back to a hidden tab with a paused sim.
                set_rate_of_time,
            )
                .chain()
                .in_set(SimulationTickSet::Last),
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
                remove_loading_resources,
                restart,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
