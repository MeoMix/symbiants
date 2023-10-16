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
        register_ant, setup_ant, teardown_ant,
        tunneling::{
            ants_add_tunnel_pheromone, ants_fade_tunnel_pheromone, ants_remove_tunnel_pheromone,
            ants_tunnel_pheromone_act, ants_tunnel_pheromone_move,
        },
        ui::{
            on_added_ant_dead, on_spawn_ant, on_update_ant_inventory, on_update_ant_orientation,
            on_update_ant_position, rerender_ant_orientation, rerender_ant_position, rerender_ant_inventory,
        },
        walk::{ants_stabilize_footing_movement, ants_walk},
    },
    background::{setup_background, teardown_background},
    common::{register_common, ui::on_add_selected},
    element::{
        register_element, setup_element, teardown_element,
        ui::{on_spawn_element, on_update_element_position, rerender_elements},
    },
    gravity::{gravity_ants, gravity_elements, gravity_stability},
    pheromone::{
        pheromone_duration_tick, register_pheromone, setup_pheromone, teardown_pheromone,
        ui::{on_spawn_pheromone, on_update_pheromone_visibility},
    },
    pointer::{handle_pointer_tap, is_pointer_captured, IsPointerCaptured},
    save::{load, save, setup_save, teardown_save},
    settings::{pre_setup_settings, register_settings, teardown_settings},
    story_state::{
        begin_story, check_story_over, continue_startup, finalize_startup, restart_story,
        StoryState,
    },
    story_time::{
        pre_setup_story_time, register_story_time, set_rate_of_time, setup_story_time,
        teardown_story_time, update_story_time, update_time_scale, StoryPlaybackState,
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
                ((
                    (
                        // It's helpful to apply gravity first because position updates are applied instantly and are seen by subsequent systems.
                        // Thus, ant actions can take into consideration where an element is this frame rather than where it was last frame.
                        gravity_elements,
                        gravity_ants,
                        // Gravity side-effects can run whenever with little difference.
                        gravity_stability,
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
                        // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                        ants_initiative,
                    )
                        .chain(),
                    check_story_over,
                    update_story_time,
                    set_rate_of_time,
                )
                    .chain())
                .run_if(not(in_state(StoryPlaybackState::Paused))),
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
                (
                    on_update_ant_position.run_if(in_state(StoryPlaybackState::Playing)),
                    on_update_ant_orientation.run_if(in_state(StoryPlaybackState::Playing)),
                    on_added_ant_dead,
                    on_update_ant_inventory.run_if(in_state(StoryPlaybackState::Playing)),
                    on_update_element_position.run_if(in_state(StoryPlaybackState::Playing)),
                    on_update_pheromone_visibility,
                    on_spawn_ant,
                    on_spawn_element,
                    on_spawn_pheromone,
                    on_add_selected,
                )
                    .chain(),
            )
                .run_if(in_state(StoryState::Telling))
                .chain(),
        );

        app.add_systems(
            OnExit(StoryPlaybackState::FastForwarding),
            (
                rerender_ant_position,
                rerender_ant_orientation,
                rerender_ant_inventory,
                rerender_elements,
            )
                .chain(),
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

#[derive(Resource)]
struct ElementSpriteHandles {
    dirt: HandleUntyped,
    food: HandleUntyped,
    sand: HandleUntyped,
}

#[derive(Resource)]
pub struct SpriteSheets {
    pub food: Handle<TextureAtlas>,
    pub dirt: Handle<TextureAtlas>,
    pub sand: Handle<TextureAtlas>,
}

fn load_textures(asset_server: Res<AssetServer>, mut commands: Commands) {
    // NOTE: `asset_server.load_folder() isn't supported in WASM`
    let sheet_dirt_asset = asset_server.load_untyped("textures/element/sheet_dirt.png");
    let sheet_sand_asset = asset_server.load_untyped("textures/element/sheet_sand.png");
    let sheet_food_asset = asset_server.load_untyped("textures/element/sheet_food.png");

    commands.insert_resource(ElementSpriteHandles {
        dirt: sheet_dirt_asset,
        food: sheet_food_asset,
        sand: sheet_sand_asset,
    });
}

fn check_textures(
    mut next_state: ResMut<NextState<StoryState>>,
    element_sprite_handles: ResMut<ElementSpriteHandles>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
) {
    let sheet_ids = [
        element_sprite_handles.dirt.id(),
        element_sprite_handles.food.id(),
        element_sprite_handles.sand.id(),
    ];

    if let LoadState::Loaded = asset_server.get_group_load_state(sheet_ids) {
        let dirt_texture_atlas_handle = add_texture_atlas_handle(
            element_sprite_handles.dirt.typed_weak(),
            &mut texture_atlases,
        );

        let food_texture_atlas_handle = add_texture_atlas_handle(
            element_sprite_handles.food.typed_weak(),
            &mut texture_atlases,
        );

        let sand_texture_atlas_handle = add_texture_atlas_handle(
            element_sprite_handles.sand.typed_weak(),
            &mut texture_atlases,
        );

        commands.insert_resource(SpriteSheets {
            dirt: dirt_texture_atlas_handle,
            food: food_texture_atlas_handle,
            sand: sand_texture_atlas_handle,
        });

        next_state.set(StoryState::LoadingSave);
    }
}

fn add_texture_atlas_handle(
    texture_handle: Handle<Image>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    // BUG: https://github.com/bevyengine/bevy/issues/1949
    // Need to use too small tile size + padding to prevent bleeding into adjacent sprites on sheet.
    texture_atlases.add(TextureAtlas::from_grid(
        texture_handle,
        Vec2::splat(126.0),
        1,
        16,
        Some(Vec2::splat(2.0)),
        None,
    ))
}
