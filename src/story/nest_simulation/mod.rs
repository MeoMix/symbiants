mod background;
pub mod gravity;
pub mod nest;

use bevy::{
    app::{MainScheduleOrder, RunFixedUpdateLoop},
    ecs::schedule::ScheduleLabel,
    prelude::*,
    utils::HashMap,
};

use crate::{
    app_state::{
        begin_story, check_story_over, continue_startup, finalize_startup, restart, AppState,
    },
    save::{load, save, setup_save, teardown_save},
    settings::{register_settings, setup_settings, teardown_settings},
    story::{
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
            nesting::{ants_nesting_action, ants_nesting_movement, register_nesting},
            register_ant,
            sleep::{ants_sleep, ants_sleep_emote, ants_wake},
            teardown_ant,
            tunneling::{
                ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone,
                ants_remove_tunnel_pheromone, ants_tunnel_pheromone_act,
                ants_tunnel_pheromone_move,
            },
            ui::{
                on_added_ant_dead, on_added_ant_emote, on_despawn_ant, on_removed_emote,
                on_spawn_ant, on_tick_emote, on_update_ant_color, on_update_ant_inventory,
                on_update_ant_orientation, on_update_ant_position, rerender_ants,
            },
            walk::{ants_stabilize_footing_movement, ants_walk},
        },
        common::{register_common, ui::on_update_selected},
        element::{register_element, teardown_element, ui::rerender_elements},
        pheromone::{
            pheromone_duration_tick, register_pheromone, setup_pheromone, teardown_pheromone,
            ui::{on_spawn_pheromone, on_update_pheromone_visibility, rerender_pheromones},
        },
        pointer::external_event::process_external_event,
        pointer::{handle_pointer_tap, is_pointer_captured, setup_pointer},
        story_time::{
            pre_setup_story_time, register_story_time, set_rate_of_time, setup_story_time,
            teardown_story_time, update_story_elapsed_ticks, update_story_real_world_time,
            update_time_scale, StoryPlaybackState, StoryTime, DEFAULT_TICKS_PER_SECOND,
        },
    },
};

use self::{
    background::{setup_background, teardown_background, update_sky_background},
    gravity::{
        gravity_ants, gravity_elements, gravity_mark_stable, gravity_mark_unstable,
        gravity_set_stability, register_gravity,
    },
    nest::{
        register_nest, setup_nest, setup_nest_ants, setup_nest_elements, setup_nest_grid,
        teardown_nest,
        ui::{
            on_added_at_nest, on_added_nest_visible_grid, on_nest_removed_visible_grid,
            on_spawn_nest,
        },
        Nest,
    },
};

use super::{
    ant::nesting::ants_nesting_start,
    common::{ui::{on_update_selected_position, ModelViewEntityMap}, setup_common, teardown_common},
    crater_simulation::crater::{
        register_crater,
        ui::{on_added_at_crater, on_added_crater_visible_grid, on_crater_removed_visible_grid},
    },
    element::{
        denormalize_element,
        ui::{
            check_element_sprite_sheet_loaded, on_despawn_element, on_update_element,
            start_load_element_sprite_sheet, update_element_exposure,
        },
    },
    grid::VisibleGridState,
    pheromone::ui::on_despawn_pheromone,
    simulation_timestep::{run_simulation_update_schedule, SimulationTime},
};

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct RunSimulationUpdateLoop;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct SimulationUpdate;

pub struct NestSimulationPlugin;

impl Plugin for NestSimulationPlugin {
    fn build(&self, app: &mut App) {
        // TODO: timing of this is weird/important, want to have schedule setup early
        app.init_resource::<SimulationTime>();
        app.add_systems(PreStartup, insert_simulation_schedule);

        app.add_systems(
            OnEnter(AppState::BeginSetup),
            (
                register_settings,
                register_common,
                register_story_time,
                register_nesting,
                register_birthing,
                register_element,
                register_gravity,
                register_ant,
                register_pheromone,
                register_nest,
                register_crater,
                start_load_element_sprite_sheet,
            )
                .chain(),
        );

        app.add_systems(OnEnter(AppState::TryLoadSave), load.pipe(continue_startup));

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (
                (setup_settings, apply_deferred).chain(),
                (
                    (setup_nest, apply_deferred).chain(),
                    (setup_nest_elements, apply_deferred).chain(),
                    (setup_nest_ants, apply_deferred).chain(),
                )
                    .chain(),
                finalize_startup,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (pre_setup_story_time, apply_deferred).chain(),
                (setup_nest_grid, apply_deferred).chain(),
                (setup_pointer, apply_deferred).chain(),
                (setup_pheromone, apply_deferred).chain(),
                (setup_background, apply_deferred).chain(),
                (setup_common, apply_deferred).chain(),
                setup_save,
                begin_story,
            )
                .chain(),
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
            update_time_scale.run_if(
                in_state(AppState::TellStory)
                    .and_then(not(in_state(StoryPlaybackState::FastForwarding))),
            ),
        );

        app.add_systems(
            Update,
            check_element_sprite_sheet_loaded.run_if(in_state(AppState::BeginSetup)),
        );

        app.init_schedule(RunSimulationUpdateLoop);
        app.add_systems(RunSimulationUpdateLoop, run_simulation_update_schedule);

        app.add_systems(
            SimulationUpdate,
            (
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
                            ants_sleep_emote.run_if(
                                resource_exists::<StoryTime>()
                                    .and_then(tick_count_elapsed(DEFAULT_TICKS_PER_SECOND)),
                            ),
                            on_tick_emote,
                            apply_deferred,
                        )
                            .chain(),
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

        // Declare all rendering systems within Update. No need to chain systems because all rendering systems
        // depend on simulation state which is updated within FixedUpdate.
        // IMPORTANT:
        // RemovedComponents<T> may contain stale/duplicate information when queried within Update
        // This occurs because the FixedUpdate schedule may run multiple times before yielding to Update
        app.add_systems(
            Update,
            (
                // Spawn
                (on_spawn_nest, on_spawn_ant, on_spawn_pheromone),
                // Despawn
                (on_despawn_ant, on_despawn_element, on_despawn_pheromone),
                // Added
                (
                    on_added_ant_dead,
                    on_added_ant_emote,
                    on_added_at_nest,
                    on_added_at_crater,
                    on_added_nest_visible_grid,
                ),
                // Removed
                (on_removed_emote, on_nest_removed_visible_grid),
                // Updated
                (
                    on_update_selected,
                    on_update_selected_position,
                    on_update_ant_position,
                    on_update_ant_orientation,
                    on_update_ant_color,
                    on_update_ant_inventory,
                    on_update_element,
                    on_update_pheromone_visibility,
                ),
            )
                .run_if(
                    in_state(AppState::TellStory).and_then(
                        not(in_state(StoryPlaybackState::FastForwarding))
                            .and_then(in_state(VisibleGridState::Nest)),
                    ),
                ),
        );

        app.add_systems(
            OnExit(StoryPlaybackState::FastForwarding),
            (rerender_ants, rerender_elements, rerender_pheromones)
                .chain()
                .run_if(in_state(VisibleGridState::Nest)),
        );

        app.add_systems(
            OnEnter(VisibleGridState::Nest),
            (rerender_ants, rerender_elements, rerender_pheromones)
                .chain()
                .run_if(in_state(StoryPlaybackState::Playing)),
        );

        app.add_systems(
            Update,
            update_story_real_world_time.run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            Update,
            update_sky_background.run_if(
                // `update_sky_background` is a view concern, and kinda heavy, so skip doing it while fast-forwarding.
                // It has some local state within it which needs to be reset when clicking "Reset Sandbox" so need to run in initializing, too.
                not(in_state(StoryPlaybackState::FastForwarding))
                    .and_then(in_state(AppState::TellStory).or_else(in_state(AppState::Cleanup))),
            ),
        );

        // Saving in WASM writes to local storage which requires dedicated support.
        app.add_systems(
            PostUpdate,
            // Saving is an expensive operation. Skip while fast-forwarding for performance.
            save.run_if(
                in_state(AppState::TellStory).and_then(in_state(StoryPlaybackState::Playing)),
            ),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                (
                    teardown_story_time,
                    teardown_settings,
                    teardown_background,
                    teardown_ant,
                    teardown_element,
                    teardown_pheromone,
                    teardown_nest,
                    teardown_save,
                    teardown_common,
                    restart,
                )
                    .chain(),
                apply_deferred,
                // TODO: maybe put this in OnExit
                // Sanity check to confirm that views were all cleaned up
                (|model_view_entity_map: Res<ModelViewEntityMap>| {
                    if model_view_entity_map.0.len() > 0 {
                        panic!(
                            "ModelViewEntityMap has {} entries remaining after cleanup",
                            model_view_entity_map.0.len()
                        );
                    }
                }),
            )
                .chain(),
        );
    }
}

// TODO: Maybe do this according to time rather than number of ticks elapsing to keep things consistent
fn tick_count_elapsed(ticks: isize) -> impl FnMut(Local<isize>, Res<StoryTime>) -> bool {
    move |mut last_run_tick_count: Local<isize>, story_time: Res<StoryTime>| {
        if *last_run_tick_count + ticks <= story_time.elapsed_ticks() {
            *last_run_tick_count = story_time.elapsed_ticks();
            true
        } else {
            false
        }
    }
}

pub fn insert_simulation_schedule(mut main_schedule_order: ResMut<MainScheduleOrder>) {
    main_schedule_order.insert_after(RunFixedUpdateLoop, RunSimulationUpdateLoop);
}
