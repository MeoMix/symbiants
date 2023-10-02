use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use bevy_turborand::GlobalRng;

use crate::{
    ant::{
        act::ants_act,
        ants_initiative,
        birthing::{ants_birthing, register_birthing},
        hunger::ants_hunger,
        nest_expansion::ants_nest_expansion,
        nesting::{ants_nesting_action, ants_nesting_movement, register_nesting},
        register_ant, setup_ant, teardown_ant,
        ui::{
            on_spawn_ant, on_update_ant_dead, on_update_ant_inventory, on_update_ant_orientation,
            on_update_ant_position,
        },
        walk::{ants_stabilize_footing_movement, ants_walk},
    },
    background::{setup_background, teardown_background},
    common::register_common,
    element::{
        register_element, setup_element, teardown_element,
        ui::{on_spawn_element, on_update_element_position},
    },
    gravity::{gravity_ants, gravity_elements, gravity_stability},
    pheromone::{
        ants_chamber_pheromone, ants_chamber_pheromone_act, ants_tunnel_pheromone,
        ants_tunnel_pheromone_act, ants_tunnel_pheromone_move, register_pheromone, setup_pheromone,
        ui::on_spawn_pheromone,
    },
    pointer::{handle_pointer_tap, is_pointer_captured, IsPointerCaptured},
    settings::{pre_setup_settings, register_settings, teardown_settings},
    story_state::{check_story_over, on_story_cleanup, StoryState},
    story_time::{
        pre_setup_story_time, register_story_time, set_rate_of_time, setup_story_time,
        teardown_story_time, update_story_time, update_time_scale, StoryPlaybackState,
    },
    world_map::{
        save::{load_existing_world, periodic_save_world_state, setup_save, teardown_save},
        setup_world_map, teardown_world_map,
    },
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveableRegistry>();

        // Some resources should be available for the entire lifetime of the application.
        // For example, IsPointerCaptured is a UI resource which is useful when interacting with the GameStart menu.
        app.init_resource::<IsPointerCaptured>();
        // TODO: I put very little thought into initializing this resource always vs saving/loading the seed.
        app.init_resource::<GlobalRng>();

        app.add_state::<StoryState>();
        // TODO: call this in setup_story_time?
        app.add_state::<StoryPlaybackState>();
        info!("adding story playback state");

        app.add_systems(
            OnEnter(StoryState::Initializing),
            (
                register_settings,
                register_common,
                register_story_time,
                register_nesting,
                register_birthing,
                register_element,
                register_ant,
                register_pheromone,
                (pre_setup_settings, apply_deferred).chain(),
                (pre_setup_story_time, apply_deferred).chain(),
                try_load_from_save,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(StoryState::Creating),
            ((setup_element, setup_ant), finalize_startup).chain(),
        );

        app.add_systems(
            OnEnter(StoryState::FinalizingStartup),
            (
                (setup_world_map, apply_deferred).chain(),
                (setup_pheromone, apply_deferred).chain(),
                (setup_background, apply_deferred).chain(),
                #[cfg(target_arch = "wasm32")]
                setup_save,
                begin_story,
            )
                .chain(),
        );

        // IMPORTANT: setup_story_time sets FixedTime.accumulated which is reset when transitioning between schedules.
        // If this is ran OnEnter FinalizingStartup then the accumulated time will be reset to zero before FixedUpdate runs.
        app.add_systems(OnExit(StoryState::FinalizingStartup), setup_story_time);

        // NOTE: don't process user input events in FixedUpdate because events in FixedUpdate are broken (should be fixed in bevy 0.12)
        app.add_systems(
            Update,
            // TODO: coupling... need to handle clicking the simulation after menus so pointer capture works properly
            (is_pointer_captured, handle_pointer_tap)
                .run_if(in_state(StoryState::Telling))
                .chain(),
        );

        app.add_systems(
            Update,
            update_time_scale.run_if(in_state(StoryState::Telling)),
        );

        app.add_systems(
            FixedUpdate,
            (
                (
                    (
                        // It's helpful to apply gravity first because position updates are applied instantly and are seen by subsequent systems.
                        // Thus, ant actions can take into consideration where an element is this frame rather than where it was last frame.
                        gravity_elements,
                        gravity_ants,
                        // Gravity side-effects can run whenever with little difference.
                        gravity_stability,
                    )
                        .chain(),
                    (
                        // Apply specific ant actions in priority order because ants take a maximum of one action per tick.
                        // An ant should not starve to hunger due to continually choosing to dig a tunnel, etc.
                        ants_stabilize_footing_movement,
                        // TODO: I'm just aggressively applying deferred until something like https://github.com/bevyengine/bevy/pull/9822 lands
                        (ants_hunger, apply_deferred).chain(),
                        (ants_birthing, apply_deferred).chain(),
                        (
                            // Apply Nesting Logic
                            ants_nesting_movement,
                            ants_nesting_action,
                            apply_deferred,
                        )
                            .chain(),
                        (ants_nest_expansion, apply_deferred).chain(),
                        (
                            // Apply Pheromones
                            ants_tunnel_pheromone,
                            ants_chamber_pheromone,
                            apply_deferred,
                        )
                            .chain(),
                        (
                            // Apply Tunneling Logic
                            ants_tunnel_pheromone_move,
                            ants_tunnel_pheromone_act,
                            apply_deferred,
                        )
                            .chain(),
                        (
                            // Apply Chambering Logic
                            // TODO: ants_chamber_pheromone_move
                            ants_chamber_pheromone_act,
                            apply_deferred,
                        )
                            .chain(),
                        // Ants move before acting because positions update instantly, but actions use commands to mutate the world and are deferred + batched.
                        // By applying movement first, commands do not need to anticipate ants having moved, but the opposite would not be true.
                        (ants_walk, ants_act, apply_deferred).chain(),
                        // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                        ants_initiative,
                    )
                        .chain(),
                    check_story_over,
                )
                    .chain(),
                // Bevy doesn't have support for PreUpdate/PostUpdate lifecycle from within FixedUpdate.
                // In an attempt to simulate this behavior, manually call `apply_deferred` because this would occur
                // when moving out of the Update stage and into the PostUpdate stage.
                // This is an important action which prevents panics while maintaining simpler code.
                // Without this, an Element might be spawned, and then despawned, with its initial render command still enqueued.
                // This would result in a panic due to missing Element entity unless the render command was rewritten manually
                // to safeguard against missing entity at time of command application.
                apply_deferred,
                // Provide an opportunity to write world state to disk.
                // This system does not run every time because saving is costly, but it does run periodically, rather than simply JIT,
                // to avoid losing too much state in the event of a crash.
                #[cfg(target_arch = "wasm32")]
                periodic_save_world_state,
                // Ensure render state reflects simulation state after having applied movements and command updates.
                (
                    on_update_ant_position,
                    on_update_ant_orientation,
                    on_update_ant_dead,
                    on_update_ant_inventory,
                    on_update_element_position,
                    on_spawn_ant,
                    on_spawn_element,
                    on_spawn_pheromone,
                )
                    .chain(),
                update_story_time,
                set_rate_of_time,
            )
                .run_if(
                    in_state(StoryState::Telling).and_then(in_state(StoryPlaybackState::Playing)),
                )
                .chain(),
        );

        app.add_systems(
            OnEnter(StoryState::Cleanup),
            (
                teardown_story_time,
                teardown_settings,
                teardown_background,
                teardown_ant,
                teardown_element,
                teardown_world_map,
                #[cfg(target_arch = "wasm32")]
                teardown_save,
                on_story_cleanup,
            )
                .chain(),
        );
    }
}

pub fn try_load_from_save(world: &mut World) {
    let is_loaded = load_existing_world(world);

    let mut story_state = world.resource_mut::<NextState<StoryState>>();
    if is_loaded {
        story_state.set(StoryState::FinalizingStartup);
    } else {
        story_state.set(StoryState::GatheringSettings);
    }
}

pub fn finalize_startup(mut story_state: ResMut<NextState<StoryState>>) {
    story_state.set(StoryState::FinalizingStartup);
}

pub fn begin_story(mut story_state: ResMut<NextState<StoryState>>) {
    story_state.set(StoryState::Telling);
}
