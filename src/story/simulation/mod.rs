pub mod common;
pub mod crater_simulation;
pub mod external_event;
pub mod nest_simulation;
pub mod settings;

use bevy::{
    app::{MainScheduleOrder, RunFixedUpdateLoop},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};

use crate::{
    app_state::{
        begin_story, check_story_over, continue_startup, finalize_startup, restart, AppState,
    },
    save::{
        bind_save_onbeforeunload, delete_save_file, initialize_save_resources, load,
        remove_save_resources, save, unbind_save_onbeforeunload,
    },
    story::{
        pointer::{handle_pointer_tap, initialize_pointer_resources, is_pointer_captured},
        simulation::{
            common::{despawn_model, register_common},
            nest_simulation::{
                ant::{
                    ants_initiative,
                    birthing::{ants_birthing, register_birthing},
                    chambering::{
                        ants_add_chamber_pheromone, ants_chamber_pheromone_act,
                        ants_fade_chamber_pheromone, ants_remove_chamber_pheromone,
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
                        ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone,
                        ants_remove_tunnel_pheromone, ants_tunnel_pheromone_act,
                        ants_tunnel_pheromone_move,
                    },
                    walk::{ants_stabilize_footing_movement, ants_walk},
                    Ant, AntAteFoodEvent,
                },
                element::{
                    denormalize_element, register_element, update_element_exposure, Element,
                },
                pheromone::{
                    initialize_pheromone_resources, pheromone_duration_tick, register_pheromone,
                    remove_pheromone_resources, Pheromone,
                },
            },
            settings::{
                initialize_settings_resources, register_settings, remove_settings_resources,
            },
        },
        story_time::{
            initialize_story_time_resources, register_story_time, remove_story_time_resources,
            set_rate_of_time, setup_story_time, update_story_elapsed_ticks,
            update_story_real_world_time, update_time_scale, StoryPlaybackState,
        },
    },
};

use self::{
    crater_simulation::crater::{spawn_crater, Crater},
    external_event::{
        initialize_external_event_resources, process_external_event,
        remove_external_event_resources,
    },
    nest_simulation::{
        gravity::{
            gravity_ants, gravity_elements, gravity_mark_stable, gravity_mark_unstable,
            gravity_set_stability, register_gravity,
        },
        nest::{
            insert_nest_grid, register_nest, spawn_nest, spawn_nest_ants, spawn_nest_elements, Nest,
        },
    },
};

use super::{
    pointer::remove_pointer_resources,
    simulation::crater_simulation::crater::register_crater,
    simulation_timestep::{run_simulation_update_schedule, SimulationTime},
};

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct RunSimulationUpdateLoop;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct SimulationUpdate;

// TODO: I'm not absolutely convinced these are good practice. It feels like this is competing with AppState transition.
// An alternative would be to have an AppState for "SimulationFinishSetup" and "RenderingFinishSetup"
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Hash)]
pub enum FinishSetupSet {
    BeforeSimulationFinishSetup,
    SimulationFinishSetup,
    AfterSimulationFinishSetup,
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Hash)]
pub enum CleanupSet {
    BeforeSimulationCleanup,
    SimulationCleanup,
    AfterSimulationCleanup,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: timing of this is weird/important, want to have schedule setup early
        // TODO: I think this should be above simulation code rather than in common?
        app.init_resource::<SimulationTime>();
        app.add_systems(PreStartup, insert_simulation_schedule);
        app.init_schedule(RunSimulationUpdateLoop);
        app.add_systems(RunSimulationUpdateLoop, run_simulation_update_schedule);

        app.configure_sets(
            OnEnter(AppState::FinishSetup),
            (
                FinishSetupSet::BeforeSimulationFinishSetup,
                FinishSetupSet::SimulationFinishSetup,
                FinishSetupSet::AfterSimulationFinishSetup,
            )
                .chain(),
        );

        app.configure_sets(
            OnEnter(AppState::Cleanup),
            (
                CleanupSet::BeforeSimulationCleanup,
                CleanupSet::SimulationCleanup,
                CleanupSet::AfterSimulationCleanup,
            )
                .chain(),
        );

        build_nest_systems(app);
        build_crater_systems(app);
        build_common_systems(app);
    }
}

pub fn insert_simulation_schedule(mut main_schedule_order: ResMut<MainScheduleOrder>) {
    main_schedule_order.insert_after(RunFixedUpdateLoop, RunSimulationUpdateLoop);
}

fn build_nest_systems(app: &mut App) {
    // TODO: This isn't a good home for this. Need to create a view-specific layer and initialize it there.
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
            .before(begin_story)
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
            .run_if(in_state(AppState::TellStory))
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

fn build_crater_systems(app: &mut App) {
    app.add_systems(OnEnter(AppState::BeginSetup), register_crater);

    app.add_systems(
        OnEnter(AppState::CreateNewStory),
        (((spawn_crater, apply_deferred).chain(),)
            .chain()
            .after(initialize_save_resources)
            .before(finalize_startup),),
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (despawn_model::<Crater>,).in_set(CleanupSet::SimulationCleanup),
    );
}

fn build_common_systems(app: &mut App) {
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
            (initialize_pointer_resources, apply_deferred).chain(),
            (initialize_external_event_resources, apply_deferred).chain(),
            // TODO: Feels weird to say saving is part of the simulation logic.
            bind_save_onbeforeunload,
            begin_story,
        )
            .chain()
            .in_set(FinishSetupSet::SimulationFinishSetup),
    );

    // IMPORTANT: setup_story_time sets FixedTime.accumulated which is reset when transitioning between schedules.
    // If this is ran OnEnter FinishSetup then the accumulated time will be reset to zero before FixedUpdate runs.
    app.add_systems(OnExit(AppState::FinishSetup), setup_story_time);

    // IMPORTANT: don't process user input in FixedUpdate/SimulationUpdate because event reads can be missed
    // https://github.com/bevyengine/bevy/issues/7691
    app.add_systems(
        Update,
        (is_pointer_captured, handle_pointer_tap)
            .run_if(in_state(AppState::TellStory))
            .chain(),
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
        save.run_if(in_state(AppState::TellStory).and_then(in_state(StoryPlaybackState::Playing))),
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (
            unbind_save_onbeforeunload,
            delete_save_file,
            remove_story_time_resources,
            remove_settings_resources,
            remove_save_resources,
            remove_pointer_resources,
            remove_external_event_resources,
            restart,
        )
            .in_set(CleanupSet::SimulationCleanup),
    );
}
