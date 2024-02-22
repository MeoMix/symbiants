pub mod ant;
pub mod gravity;
pub mod nest;

use crate::common::{
    ant::Ant, element::Element, grid::ElementEntityPositionCache, pheromone::{
        initialize_pheromone_resources, decay_pheromone_strength, remove_pheromone_resources,
        Pheromone,
    }
};

use self::{
    ant::{
        birthing::{ants_birthing, register_birthing},
        chambering::{
            ants_add_chamber_pheromone, ants_chamber_pheromone_act, ants_fade_chamber_pheromone,
            ants_remove_chamber_pheromone,
        },
        dig::ants_dig,
        drop::ants_drop,
        nest_expansion::ants_nest_expansion,
        nesting::{
            ants_nesting_action, ants_nesting_movement, ants_nesting_start, register_nesting,
        },
        register_ant,
        sleep::{ants_sleep, ants_wake},
        travel::ants_travel_to_crater,
        tunneling::{
            ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone, ants_remove_tunnel_pheromone,
            ants_tunnel_pheromone_act, ants_tunnel_pheromone_move,
        },
        walk::{ants_stabilize_footing_movement, ants_walk},
    },
    gravity::{
        gravity_ants, gravity_elements, gravity_mark_stable, gravity_mark_unstable,
        gravity_set_stability, register_gravity,
    },
    nest::{
        insert_nest_grid, register_nest, spawn_nest, spawn_nest_ants, spawn_nest_elements, AtNest,
        Nest,
    },
};
use super::{
    despawn_model, settings::initialize_settings_resources, AppState, CleanupSet, FinishSetupSet,
    SimulationTickSet, StoryPlaybackState,
};
use bevy::prelude::*;

pub struct NestSimulationPlugin;

impl Plugin for NestSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                register_nesting,
                register_birthing,
                register_gravity,
                register_ant,
                register_nest,
            ),
        );

        app.add_systems(
            OnExit(AppState::MainMenu),
            (
                // Call `apply_deferred` to ensure Settings (via `initialize_settings_resources`) is available for use.
                apply_deferred,
                spawn_nest,
                apply_deferred,
                (spawn_nest_elements, spawn_nest_ants),
            )
                .chain()
                .after(initialize_settings_resources),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                insert_nest_grid,
                apply_deferred,
                initialize_pheromone_resources::<AtNest>,
            )
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        // TODO: I'm just aggressively applying deferred until something like https://github.com/bevyengine/bevy/pull/9822 lands
        app.add_systems(
            FixedUpdate,
            (
                // TODO: Consider whether gravity is special enough to warrant being placed in PreSimulationTick
                (
                    gravity_set_stability,
                    apply_deferred,
                    // It's helpful to apply gravity first because position updates are applied instantly and are seen by subsequent systems.
                    // Thus, ant actions can take into consideration where an element is this frame rather than where it was last frame.
                    gravity_elements,
                    gravity_ants,
                    // Gravity side-effects can run whenever with little difference.
                    gravity_mark_stable,
                    gravity_mark_unstable,
                    apply_deferred,
                )
                    .chain(),
                (
                    // Apply specific ant actions in priority order because ants take a maximum of one action per tick.
                    // An ant should not starve to hunger due to continually choosing to dig a tunnel, etc.
                    ants_stabilize_footing_movement,
                    (ants_birthing, apply_deferred).chain(),
                    (ants_sleep, ants_wake, apply_deferred).chain(),
                    (
                        // Apply Nesting Logic
                        ants_nesting_start,
                        ants_nesting_movement,
                        ants_nesting_action,
                        apply_deferred,
                    )
                        .chain(),
                    (ants_nest_expansion, apply_deferred).chain(),
                    (decay_pheromone_strength::<AtNest>, apply_deferred).chain(),
                    // Tunneling Pheromone:
                    (
                        // Fade first (or last) to ensure that if movement occurs that resulting position is reflective
                        // of that tiles PheromoneStrength. If fade is applied after movement, but before action, then
                        // there will be an off-by-one between PheromoneStrength of tile being stood on and what is applied to ant.
                        ants_fade_tunnel_pheromone,
                        // Move first, then sync state with current tile, then take action reflecting current state.
                        ants_tunnel_pheromone_move,
                        // Now apply pheromone onto ant. Call apply_deferred after each to ensure remove enforces
                        // constraints immediately on any applied pheromone so move/act work on current assumptions.
                        ants_add_tunnel_pheromone,
                        apply_deferred,
                        ants_remove_tunnel_pheromone,
                        apply_deferred,
                        ants_tunnel_pheromone_act,
                        apply_deferred,
                    )
                        .chain(),
                    // Chambering Pheromone:
                    (
                        ants_fade_chamber_pheromone,
                        // TODO: ants_chamber_pheromone_move
                        ants_add_chamber_pheromone,
                        apply_deferred,
                        ants_remove_chamber_pheromone,
                        apply_deferred,
                        ants_chamber_pheromone_act,
                        apply_deferred,
                    )
                        .chain(),
                    // Ants move before acting because positions update instantly, but actions use commands to mutate the world and are deferred + batched.
                    // By applying movement first, commands do not need to anticipate ants having moved, but the opposite would not be true.
                    (
                        ants_travel_to_crater,
                        apply_deferred,
                        ants_walk,
                        ants_dig,
                        apply_deferred,
                        ants_drop,
                        apply_deferred,
                    )
                        .chain(),
                )
                    .chain(),
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
                despawn_model::<Ant, AtNest>,
                despawn_model::<Element, AtNest>,
                despawn_model::<ElementEntityPositionCache, AtNest>,
                despawn_model::<Pheromone, AtNest>,
                despawn_model::<Nest, AtNest>,
                remove_pheromone_resources::<AtNest>,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
