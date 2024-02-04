pub mod ant;
pub mod element;
pub mod gravity;
pub mod nest;
pub mod pheromone;

use self::{
    ant::{
        ants_initiative,
        birthing::{ants_birthing, register_birthing},
        chambering::{
            ants_add_chamber_pheromone, ants_chamber_pheromone_act, ants_fade_chamber_pheromone,
            ants_remove_chamber_pheromone,
        },
        death::on_ants_add_dead,
        dig::ants_dig,
        digestion::ants_digestion,
        drop::ants_drop,
        hunger::{ants_hunger_act, ants_hunger_tick, ants_regurgitate},
        nest_expansion::ants_nest_expansion,
        nesting::ants_nesting_start,
        nesting::{ants_nesting_action, ants_nesting_movement, register_nesting},
        register_ant,
        sleep::{ants_sleep, ants_wake},
        tunneling::{
            ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone, ants_remove_tunnel_pheromone,
            ants_tunnel_pheromone_act, ants_tunnel_pheromone_move,
        },
        walk::{ants_stabilize_footing_movement, ants_walk},
        Ant, AntAteFoodEvent,
    },
    element::{register_element, update_element_exposure, Element},
    gravity::{
        gravity_ants, gravity_elements, gravity_mark_stable, gravity_mark_unstable,
        gravity_set_stability, register_gravity,
    },
    nest::{
        insert_nest_grid, register_nest, spawn_nest, spawn_nest_ants, spawn_nest_elements, Nest,
    },
    pheromone::{
        initialize_pheromone_resources, pheromone_duration_tick, register_pheromone,
        remove_pheromone_resources, Pheromone,
    },
};
use super::{
    despawn_model, settings::initialize_settings_resources, AppState, CleanupSet, FinishSetupSet,
    SimulationTickSet, SimulationUpdate, StoryPlaybackState,
};
use bevy::prelude::*;

pub struct NestSimulationPlugin;

impl Plugin for NestSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AntAteFoodEvent>();

        app.add_systems(
            OnEnter(AppState::BeginSetup),
            (
                register_nesting,
                register_birthing,
                register_element,
                register_gravity,
                register_ant,
                register_pheromone,
                register_nest,
            ),
        );

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
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
                (
                    initialize_pheromone_resources,
                    // IMPORTANT:
                    // `ElementExposure` isn't persisted because it's derivable. It is required for rendering.
                    // Don't rely on `SimulationUpdate` to set `ElementExposure` because it should be possible to render
                    // the world's initial state without advancing the simulation.
                    update_element_exposure,
                ),
            )
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            SimulationUpdate,
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
                    // TODO: I'm just aggressively applying deferred until something like https://github.com/bevyengine/bevy/pull/9822 lands
                    (
                        ants_digestion,
                        ants_hunger_tick,
                        ants_hunger_act,
                        apply_deferred,
                        ants_regurgitate,
                        apply_deferred,
                    )
                        .chain(),
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
                    (pheromone_duration_tick, apply_deferred).chain(),
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
                        ants_walk,
                        ants_dig,
                        apply_deferred,
                        ants_drop,
                        apply_deferred,
                    )
                        .chain(),
                    on_ants_add_dead,
                    // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                    ants_initiative,
                )
                    .chain(),
            )
                .run_if(not(in_state(StoryPlaybackState::Paused)))
                .chain()
                .in_set(SimulationTickSet::SimulationTick),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                despawn_model::<Ant>,
                despawn_model::<Element>,
                despawn_model::<Pheromone>,
                despawn_model::<Nest>,
                remove_pheromone_resources,
            )
                .in_set(CleanupSet::SimulationCleanup),
        );
    }
}
