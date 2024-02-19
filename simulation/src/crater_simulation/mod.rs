pub mod ant;
pub mod crater;

use crate::{
    common::{
        ant::Ant,
        element::Element,
        pheromone::{
            initialize_pheromone_resources, pheromone_duration_tick, remove_pheromone_resources,
            Pheromone,
        },
    },
    story_time::StoryPlaybackState,
    SimulationTickSet,
};

use self::{
    ant::{
        dig::ants_dig, emit_pheromone::ants_emit_pheromone, register_ant,
        set_pheromone_emitter::ants_set_pheromone_emitter, travel::ants_travel_to_nest,
        walk::ants_walk,
    },
    crater::{
        register_crater, spawn_crater, spawn_crater_ants, spawn_crater_elements, AtCrater, Crater,
    },
};
use super::{
    apply_deferred, despawn_model, insert_crater_grid, settings::initialize_settings_resources,
    AppState, CleanupSet, FinishSetupSet,
};
use bevy::prelude::*;

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (register_crater, register_ant));

        app.add_systems(
            OnExit(AppState::MainMenu),
            (
                // Call `apply_deferred` to ensure Settings (via `initialize_settings_resources`) is available for use.
                apply_deferred,
                spawn_crater,
                apply_deferred,
                (spawn_crater_elements, spawn_crater_ants),
            )
                .chain()
                .after(initialize_settings_resources),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                insert_crater_grid,
                apply_deferred,
                initialize_pheromone_resources::<AtCrater>,
            )
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            FixedUpdate,
            (
                (pheromone_duration_tick::<AtCrater>, apply_deferred).chain(),
                (ants_set_pheromone_emitter, apply_deferred).chain(),
                ants_emit_pheromone,
                // Ants move before acting because positions update instantly, but actions use commands to mutate the world and are deferred + batched.
                // By applying movement first, commands do not need to anticipate ants having moved, but the opposite would not be true.
                ants_travel_to_nest,
                ants_walk,
                ants_dig,
            )
                .run_if(
                    in_state(AppState::TellStory)
                        .and_then(not(in_state(StoryPlaybackState::Paused))),
                )
                .chain()
                .in_set(SimulationTickSet::SimulationTick),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                despawn_model::<Ant, AtCrater>,
                despawn_model::<Element, AtCrater>,
                despawn_model::<Pheromone, AtCrater>,
                despawn_model::<Crater, AtCrater>,
                remove_pheromone_resources::<AtCrater>,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
