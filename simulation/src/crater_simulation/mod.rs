pub mod ant;
pub mod crater;
pub mod element;
pub mod pheromone;

use crate::{
    common::pheromone::{initialize_pheromone_resources, remove_pheromone_resources, Pheromone},
    nest_simulation::{
        ant::{ants_initiative, Ant},
        element::Element,
    },
    story_time::StoryPlaybackState,
    SimulationTickSet, SimulationUpdate,
};

use self::{
    ant::{
        dig::ants_dig, emit_pheromone::ants_emit_pheromone, register_ant,
        set_pheromone_emitter::ants_set_pheromone_emitter, walk::ants_walk,
    },
    crater::{
        register_crater, spawn_crater, spawn_crater_ants, spawn_crater_elements, AtCrater, Crater,
    },
    pheromone::pheromone_duration_tick,
};
use super::{
    apply_deferred, despawn_model, insert_crater_grid, settings::initialize_settings_resources,
    AppState, CleanupSet, FinishSetupSet,
};
use bevy::prelude::*;

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::BeginSetup),
            (register_crater, register_ant),
        );

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
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
            SimulationUpdate,
            (
                (pheromone_duration_tick, apply_deferred).chain(),
                (ants_set_pheromone_emitter, apply_deferred).chain(),
                ants_emit_pheromone,
                // Ants move before acting because positions update instantly, but actions use commands to mutate the world and are deferred + batched.
                // By applying movement first, commands do not need to anticipate ants having moved, but the opposite would not be true.
                ants_walk,
                ants_dig,
                apply_deferred,
                // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                ants_initiative::<AtCrater>,
            )
                .run_if(not(in_state(StoryPlaybackState::Paused)))
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
