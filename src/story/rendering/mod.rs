// TODO: This shouldn't need to be public I think?
pub mod common;
mod crater_rendering;
mod nest_rendering;
mod pan_zoom_camera;

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

use self::{
    common::{
        clear_selection, despawn_common_entities, despawn_view, despawn_view_by_model,
        initialize_common_resources, on_despawn, on_update_selected, on_update_selected_position,
        remove_common_resources, ModelViewEntityMap,
    },
    nest_rendering::{
        ant::{
            cleanup_ants,
            emote::{
                ants_sleep_emote, on_added_ant_emote, on_ant_ate_food, on_ant_wake_up,
                on_removed_emote, on_tick_emote,
            },
            on_added_ant_dead, on_spawn_ant, on_update_ant_color, on_update_ant_inventory,
            on_update_ant_orientation, on_update_ant_position, rerender_ants,
        },
        background::{
            cleanup_background, spawn_background, spawn_background_tilemap, update_sky_background,
            Background, BackgroundTilemap,
        },
        element::{
            cleanup_elements, on_spawn_element, on_update_element_exposure,
            on_update_element_position, rerender_elements,
            sprite_sheet::{
                check_element_sprite_sheet_loaded, start_load_element_sprite_sheet, ElementTilemap,
            },
        },
        nest::{mark_nest_hidden, mark_nest_visible},
        pheromone::{
            cleanup_pheromones, on_spawn_pheromone, on_update_pheromone_visibility,
            rerender_pheromones,
        },
    },
    pan_zoom_camera::PanZoomCameraPlugin,
};

use super::simulation::{
    app_state::AppState,
    nest_simulation::{ant::Ant, element::Element, nest::AtNest, pheromone::Pheromone},
    story_time::{StoryTime, DEFAULT_TICKS_PER_SECOND},
    CleanupSet, FinishSetupSet,
};

// TODO: Find a better home for this?
// TODO: It's weird that I have the concept of `VisibleGrid` in addition to `VisibleGridState`
// Generally representing the same state in two different ways is a great way to introduce bugs.
#[derive(States, Default, Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum VisibleGridState {
    #[default]
    Nest,
    Crater,
}

pub struct RenderingPlugin;

/// Plugin that handles rendering of the simulation. Plugin handles rendering nest and crater views, but only one view will be visible at a time.
/// IMPORTANT:
///     RemovedComponents<T> may contain stale/duplicate data when queried from within Update.
///     This occurs because the simulation may tick multiple times before rendering systems run.
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PanZoomCameraPlugin, TilemapPlugin));
        app.add_state::<VisibleGridState>();

        build_common_systems(app);
        build_nest_systems(app);
        build_crater_systems(app);
    }
}

fn build_common_systems(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::FinishSetup),
        (initialize_common_resources,).in_set(FinishSetupSet::BeforeSimulationFinishSetup),
    );

    app.add_systems(
        Update,
        (on_update_selected, on_update_selected_position).run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnExit(AppState::Cleanup),
        |model_view_entity_map: Res<ModelViewEntityMap>| {
            if model_view_entity_map.len() > 0 {
                panic!(
                    "ModelViewEntityMap has {} entries remaining after cleanup",
                    model_view_entity_map.len()
                );
            }
        },
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (despawn_common_entities, remove_common_resources)
            .in_set(CleanupSet::BeforeSimulationCleanup),
    );
}

fn build_crater_systems(_app: &mut App) {}

fn build_nest_systems(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::BeginSetup),
        start_load_element_sprite_sheet,
    );

    app.add_systems(
        Update,
        check_element_sprite_sheet_loaded.run_if(in_state(AppState::BeginSetup)),
    );

    app.add_systems(
        OnEnter(AppState::FinishSetup),
        (spawn_background_tilemap, apply_deferred, spawn_background)
            .chain()
            .in_set(FinishSetupSet::AfterSimulationFinishSetup),
    );

    app.add_systems(
        Update,
        update_sky_background.run_if(
            // TODO: `update_sky_background` uses Local<_> which needs to be reset during cleanup.
            // This is pretty hacky. Once Bevy supports removing systems it'll be easier to remove.
            in_state(AppState::TellStory).or_else(in_state(AppState::Cleanup)),
        ),
    );

    app.add_systems(
        Update,
        (
            // Spawn
            (on_spawn_ant, on_spawn_element, on_spawn_pheromone),
            // Despawn
            (
                on_despawn::<Ant, AtNest>,
                on_despawn::<Element, AtNest>,
                on_despawn::<Pheromone, AtNest>,
            ),
            // Added
            (on_added_ant_dead, on_added_ant_emote),
            // Removed
            (on_removed_emote),
            // Updated
            (
                on_update_ant_position,
                on_update_ant_orientation,
                on_update_ant_color,
                on_update_ant_inventory,
                on_ant_ate_food,
                // TODO: naming inconsistencies, but probably want to go more this direction rather than away.
                ants_sleep_emote.run_if(
                    // TODO: this feels hacky? trying to rate limit how often checks for sleeping emoting occurs.
                    resource_exists::<StoryTime>()
                        .and_then(tick_count_elapsed(DEFAULT_TICKS_PER_SECOND)),
                ),
                on_ant_wake_up,
                on_tick_emote,
                on_update_element_position,
                on_update_element_exposure,
                on_update_pheromone_visibility,
            ),
        )
            .run_if(in_state(AppState::TellStory)),
    );

    // When beginning the story, start by showing the Nest.
    app.add_systems(OnEnter(AppState::TellStory), mark_nest_visible);

    app.add_systems(
        OnEnter(VisibleGridState::Nest),
        (
            (spawn_background_tilemap, apply_deferred, spawn_background).chain(),
            rerender_ants,
            rerender_elements,
            rerender_pheromones,
            mark_nest_visible,
        )
            .run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnExit(VisibleGridState::Nest),
        (
            despawn_view::<Background>,
            despawn_view::<BackgroundTilemap>,
            despawn_view_by_model::<Ant>,
            despawn_view_by_model::<Element>,
            // TODO: Despawn ElementTilemap?
            despawn_view_by_model::<Pheromone>,
            clear_selection,
            mark_nest_hidden,
        )
            .run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (
            despawn_view::<Background>,
            despawn_view::<BackgroundTilemap>,
            cleanup_background,
            despawn_view_by_model::<Ant>,
            cleanup_ants,
            despawn_view_by_model::<Element>,
            despawn_view::<ElementTilemap>,
            cleanup_elements,
            despawn_view_by_model::<Pheromone>,
            cleanup_pheromones,
        )
            .in_set(CleanupSet::BeforeSimulationCleanup),
    );
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
