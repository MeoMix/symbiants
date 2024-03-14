pub mod ant;
pub mod crater;

use crate::{
    common::{
        ant::Ant,
        element::Element,
        grid::ElementEntityPositionCache,
        pheromone::{
            decay_pheromone_strength, initialize_pheromone_resources, remove_pheromone_resources,
            Pheromone,
        },
    },
    story_time::StoryPlaybackState,
    SimulationTickSet,
};

use self::{
    ant::{
        dig::ants_dig, emit_pheromone::ants_emit_pheromone,
        follow_pheromone::ants_follow_pheromone, register_ant,
        set_pheromone_emitter::ants_set_pheromone_emitter, travel::ants_travel_to_nest,
        wander::ants_wander,
    },
    crater::{
        emit_pheromone::{food_emit_pheromone, nest_entrance_emit_pheromone},
        register_crater, spawn_crater, spawn_crater_ants, spawn_crater_elements, AtCrater, Crater,
    },
};
use super::{
    despawn_model, insert_crater_grid, settings::initialize_settings_resources,
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
                (decay_pheromone_strength::<AtCrater>, apply_deferred).chain(),
                (ants_set_pheromone_emitter, apply_deferred).chain(),
                nest_entrance_emit_pheromone,
                food_emit_pheromone,
                ants_emit_pheromone,
                ants_travel_to_nest,
                ants_follow_pheromone,
                ants_wander,
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
                despawn_model::<ElementEntityPositionCache, AtCrater>,
                despawn_model::<Pheromone, AtCrater>,
                despawn_model::<Crater, AtCrater>,
                remove_pheromone_resources::<AtCrater>,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
