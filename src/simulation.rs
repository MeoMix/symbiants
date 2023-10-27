use bevy::{asset::LoadState, prelude::*};
use bevy_save::SaveableRegistry;
use bevy_turborand::GlobalRng;

use crate::{
    ant::{
        ants_initiative,
        birthing::{ants_birthing, register_birthing},
        chambering::{
            ants_add_chamber_pheromone, ants_chamber_pheromone_act, ants_fade_chamber_pheromone,
            ants_remove_chamber_pheromone,
        },
        dig::ants_dig,
        drop::ants_drop,
        hunger::{ants_hunger, ants_regurgitate},
        nest_expansion::ants_nest_expansion,
        nesting::{ants_nesting_action, ants_nesting_movement, register_nesting},
        register_ant, setup_ant,
        sleep::{ants_sleep, ants_sleep_emote, ants_wake},
        teardown_ant,
        tunneling::{
            ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone, ants_remove_tunnel_pheromone,
            ants_tunnel_pheromone_act, ants_tunnel_pheromone_move,
        },
        ui::{
            on_added_ant_dead, on_added_ant_emote, on_despawn_ant, on_removed_emote, on_spawn_ant,
            on_tick_emote, on_update_ant_inventory, on_update_ant_orientation,
            on_update_ant_position, rerender_ants,
        },
        walk::{ants_stabilize_footing_movement, ants_walk},
    },
    background::{setup_background, teardown_background, update_sky_background},
    common::{
        pre_setup_common, register_common, setup_common,
        ui::{on_added_selected, on_removed_selected},
    },
    element::{
        register_element, setup_element, teardown_element,
        ui::{
            on_spawn_element, on_update_element_position, rerender_elements, ElementSpriteHandles,
        },
    },
    external_event::process_external_event,
    gravity::{gravity_ants, gravity_elements, gravity_mark_stable, gravity_mark_unstable},
    pheromone::{
        pheromone_duration_tick, register_pheromone, setup_pheromone, teardown_pheromone,
        ui::{on_spawn_pheromone, on_update_pheromone_visibility, render_pheromones},
    },
    pointer::{handle_pointer_tap, is_pointer_captured, setup_pointer, IsPointerCaptured},
    save::{load, save, setup_save, teardown_save},
    settings::{pre_setup_settings, register_settings, teardown_settings},
    story_state::{
        begin_story, check_story_over, continue_startup, finalize_startup, restart_story,
        StoryState,
    },
    story_time::{
        pre_setup_story_time, register_story_time, set_rate_of_time, setup_story_time,
        teardown_story_time, update_story_elapsed_ticks, update_story_real_world_time,
        update_time_scale, StoryElapsedTicks, StoryPlaybackState, DEFAULT_TICKS_PER_SECOND,
    },
    world_map::{setup_world_map, teardown_world_map},
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
                (pre_setup_common, apply_deferred).chain(),
                load_textures,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(StoryState::LoadingSave),
            load.pipe(continue_startup),
        );

        app.add_systems(
            OnEnter(StoryState::Creating),
            ((setup_element, setup_ant), finalize_startup).chain(),
        );

        app.add_systems(
            OnEnter(StoryState::FinalizingStartup),
            (
                (setup_world_map, apply_deferred).chain(),
                (setup_common, apply_deferred).chain(),
                (setup_pointer, apply_deferred).chain(),
                (setup_pheromone, apply_deferred).chain(),
                (setup_background, apply_deferred).chain(),
                setup_save,
                begin_story,
            )
                .chain(),
        );

        // IMPORTANT: setup_story_time sets FixedTime.accumulated which is reset when transitioning between schedules.
        // If this is ran OnEnter FinalizingStartup then the accumulated time will be reset to zero before FixedUpdate runs.
        app.add_systems(OnExit(StoryState::FinalizingStartup), setup_story_time);

        // IMPORTANT: don't process user input in FixedUpdate because events in FixedUpdate are broken
        // https://github.com/bevyengine/bevy/issues/7691
        app.add_systems(
            Update,
            (is_pointer_captured, handle_pointer_tap)
                .run_if(in_state(StoryState::Telling))
                .chain(),
        );

        app.add_systems(
            Update,
            update_time_scale.run_if(
                in_state(StoryState::Telling)
                    .and_then(not(in_state(StoryPlaybackState::FastForwarding))),
            ),
        );

        app.add_systems(
            Update,
            check_textures.run_if(in_state(StoryState::Initializing)),
        );

        app.add_systems(
            FixedUpdate,
            (
                (process_external_event, apply_deferred).chain(),
                ((
                    (
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
                        (ants_hunger, ants_regurgitate, apply_deferred).chain(),
                        (ants_birthing, apply_deferred).chain(),
                        (ants_sleep, ants_wake, ants_sleep_emote, apply_deferred).chain(),
                        (
                            ants_sleep_emote.run_if(tick_count_elapsed(DEFAULT_TICKS_PER_SECOND)),
                            on_tick_emote,
                            apply_deferred,
                        )
                            .chain(),
                        (
                            // Apply Nesting Logic
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
                        // TODO: This isn't working well right now, causes panics
                        // on_ants_add_dead,
                        // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                        ants_initiative,
                    )
                        .chain(),
                    check_story_over,
                    update_story_elapsed_ticks,
                )
                    .chain())
                .run_if(not(in_state(StoryPlaybackState::Paused))),
                // real-world time should update even if the story is paused because real-world time doesn't pause
                // rate_of_time needs to run when app is paused because fixed_time accumulations need to be cleared while app is paused
                // to prevent running FixedUpdate schedule repeatedly (while no-oping) when coming back to a hidden tab with a paused sim.
                (update_story_real_world_time, set_rate_of_time).chain(),
                // Bevy doesn't have support for PreUpdate/PostUpdate lifecycle from within FixedUpdate.
                // In an attempt to simulate this behavior, manually call `apply_deferred` because this would occur
                // when moving out of the Update stage and into the PostUpdate stage.
                // This is an important action which prevents panics while maintaining simpler code.
                // Without this, an Element might be spawned, and then despawned, with its initial render command still enqueued.
                // This would result in a panic due to missing Element entity unless the render command was rewritten manually
                // to safeguard against missing entity at time of command application.
                apply_deferred,
                // Ensure render state reflects simulation state after having applied movements and command updates.
                // Must run in FixedUpdate otherwise change detection won't properly work if FixedUpdate loop runs multiple times in a row.
                // Must run even when simulation is paused to reflect user input.
                (
                    // Spawn
                    on_spawn_ant,
                    on_spawn_element,
                    on_spawn_pheromone,

                    // Despawn
                    on_despawn_ant,

                    // Added
                    on_added_ant_dead,
                    on_added_ant_emote,
                    on_added_selected,

                    // Removed
                    on_removed_selected,
                    on_removed_emote,

                    // Updated
                    on_update_ant_position,
                    on_update_ant_orientation,
                    on_update_ant_inventory,
                    on_update_element_position,
                    on_update_pheromone_visibility,
                )
                    .chain()
                    .run_if(not(in_state(StoryPlaybackState::FastForwarding))),
            )
                .run_if(in_state(StoryState::Telling))
                .chain(),
        );

        app.add_systems(
            OnExit(StoryPlaybackState::FastForwarding),
            (rerender_ants, rerender_elements, render_pheromones).chain(),
        );

        app.add_systems(
            Update,
            update_story_real_world_time.run_if(in_state(StoryState::Telling)),
        );

        app.add_systems(
            Update,
            update_sky_background.run_if(
                // `update_sky_background` is a view concern, and kinda heavy, so skip doing it while fast-forwarding.
                // It has some local state within it which needs to be reset when clicking "Reset Story" so need to run in initializing, too.
                not(in_state(StoryPlaybackState::FastForwarding)).and_then(
                    in_state(StoryState::Telling).or_else(in_state(StoryState::Initializing)),
                ),
            ),
        );

        // Saving in WASM writes to local storage which requires dedicated support.
        app.add_systems(
            PostUpdate,
            // Saving is an expensive operation. Skip while fast-forwarding for performance.
            save.run_if(
                in_state(StoryState::Telling).and_then(in_state(StoryPlaybackState::Playing)),
            ),
        );

        app.add_systems(
            OnEnter(StoryState::Cleanup),
            (
                teardown_story_time,
                teardown_settings,
                teardown_background,
                teardown_ant,
                teardown_element,
                teardown_pheromone,
                teardown_world_map,
                teardown_save,
                restart_story,
            )
                .chain(),
        );
    }
}

fn load_textures(asset_server: Res<AssetServer>, mut commands: Commands) {
    // NOTE: `asset_server.load_folder() isn't supported in WASM`
    // BUG: https://github.com/bevyengine/bevy/issues/1949
    // Intentionally not using SpriteSheet/TextureAtlas due to subpixel rounding causing bleed artifacts.
    let dirt_handles = (0..=15)
        .map(|i| format!("textures/element/dirt/{}.png", i))
        .map(|path| asset_server.load_untyped(path))
        .collect();

    let sand_handles = (0..=15)
        .map(|i| format!("textures/element/sand/{}.png", i))
        .map(|path| asset_server.load_untyped(path))
        .collect();

    let food_handles = (0..=15)
        .map(|i| format!("textures/element/food/{}.png", i))
        .map(|path| asset_server.load_untyped(path))
        .collect();

    let air_handle = asset_server.load_untyped("textures/element/air/air.png");

    commands.insert_resource(ElementSpriteHandles {
        dirt: dirt_handles,
        sand: sand_handles,
        food: food_handles,
        air: air_handle,
    });
}

fn check_textures(
    mut next_state: ResMut<NextState<StoryState>>,
    element_sprite_handles: ResMut<ElementSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(element_sprite_handles.handle_ids())
    {
        next_state.set(StoryState::LoadingSave);
    }
}

// TODO: Maybe do this according to time rather than number of ticks elapsing to keep things consistent
fn tick_count_elapsed(ticks: isize) -> impl FnMut(Local<isize>, Res<StoryElapsedTicks>) -> bool {
    move |mut last_run_tick_count: Local<isize>, elapsed_ticks: Res<StoryElapsedTicks>| {
        if *last_run_tick_count + ticks <= elapsed_ticks.value() {
            *last_run_tick_count = elapsed_ticks.value();
            true
        } else {
            false
        }
    }
}
