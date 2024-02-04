pub mod ant;
pub mod element;
pub mod gravity;
pub mod nest;
pub mod pheromone;

use crate::{
    app_state::check_story_over,
    story_time::{update_story_elapsed_ticks, update_story_real_world_time},
};

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
    element::{denormalize_element, register_element, update_element_exposure, Element},
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
    apply_deferred, despawn_model, finalize_startup, initialize_save_resources,
    post_setup_clear_change_detection, process_external_event, set_rate_of_time, AppState,
    CleanupSet, FinishSetupSet, SimulationUpdate, StoryPlaybackState,
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
            ((
                (spawn_nest, apply_deferred).chain(),
                (spawn_nest_elements, apply_deferred).chain(),
                (spawn_nest_ants, apply_deferred).chain(),
            )
                .chain()
                .after(initialize_save_resources)
                .before(finalize_startup),),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (insert_nest_grid, apply_deferred).chain(),
                (initialize_pheromone_resources, apply_deferred).chain(),
                // IMPORTANT:
                // `ElementExposure` isn't persisted because it's derivable. It is required for rendering.
                // Don't rely on `SimulationUpdate` to set `ElementExposure` because it should be possible to render
                // the world's initial state without advancing the simulation.
                (update_element_exposure, apply_deferred).chain(),
            )
                .chain()
                .before(post_setup_clear_change_detection)
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            SimulationUpdate,
            (
                // TODO: process_external_event is common not nest.
                (process_external_event, apply_deferred).chain(),
                (denormalize_element, apply_deferred).chain(),
                ((
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
                    // TODO: These are common not nest
                    check_story_over,
                    update_story_elapsed_ticks,
                )
                    .chain())
                .run_if(not(in_state(StoryPlaybackState::Paused))),
                // If this doesn't run then when user spawns elements they won't gain exposure if simulation is paused.
                apply_deferred,
                update_element_exposure,
                // real-world time should update even if the story is paused because real-world time doesn't pause
                // rate_of_time needs to run when app is paused because fixed_time accumulations need to be cleared while app is paused
                // to prevent running FixedUpdate schedule repeatedly (while no-oping) when coming back to a hidden tab with a paused sim.
                (update_story_real_world_time, set_rate_of_time).chain(),
            )
                .chain(),
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
