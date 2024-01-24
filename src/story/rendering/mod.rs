// TODO: This shouldn't need to be public I think?
pub mod common;
mod crater_rendering;
mod nest_rendering;
pub mod pan_zoom_camera;

use bevy::prelude::*;

use crate::app_state::AppState;

use self::{
    common::{
        despawn_common_entities, despawn_view, initialize_common_resources, on_update_selected,
        on_update_selected_position, remove_common_resources, ModelViewEntityMap,
    },
    nest_rendering::{
        ant::{
            ants_sleep_emote, cleanup_ants, on_added_ant_dead, on_added_ant_emote, on_ant_ate_food,
            on_ant_wake_up, on_despawn_ant, on_removed_emote, on_spawn_ant, on_tick_emote,
            on_update_ant_color, on_update_ant_inventory, on_update_ant_orientation,
            on_update_ant_position, rerender_ants,
        },
        element::{
            check_element_sprite_sheet_loaded, cleanup_elements, on_despawn_element,
            on_update_element, rerender_elements, start_load_element_sprite_sheet,
        },
        nest::on_spawn_nest,
        pheromone::{
            cleanup_pheromones, on_despawn_pheromone, on_spawn_pheromone,
            on_update_pheromone_visibility, rerender_pheromones,
        },
    },
    pan_zoom_camera::PanZoomCameraPlugin,
};

use super::{
    ant::Ant,
    element::Element,
    grid::VisibleGridState,
    pheromone::Pheromone,
    simulation::CleanupSet,
    story_time::{StoryTime, DEFAULT_TICKS_PER_SECOND},
};

pub struct RenderingPlugin;

/// Plugin that handles rendering of the simulation. Plugin handles rendering nest and crater views, but only one view will be visible at a time.
/// IMPORTANT:
///     RemovedComponents<T> may contain stale/duplicate data when queried from within Update.
///     This occurs because the simulation may tick multiple times before rendering systems run.
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PanZoomCameraPlugin));

        build_common_systems(app);
        build_nest_systems(app);
        build_crater_systems(app);
    }
}

fn build_common_systems(app: &mut App) {
    app.add_systems(OnEnter(AppState::FinishSetup), initialize_common_resources);

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
        Update,
        (
            // TODO: This apply_deferred sucks but I'm relying on view state to reactively render
            // and so I need this to be accurate now not next frame.
            on_spawn_nest,
            apply_deferred,
            // Spawn
            (on_spawn_ant, on_spawn_pheromone),
            // Despawn
            // TODO: make these generic
            (on_despawn_ant, on_despawn_element, on_despawn_pheromone),
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
                on_update_element,
                on_update_pheromone_visibility,
            ),
        )
            // TODO: This .chain() isn't desirable, but is necessary due to use of `apply_deferred` above.
            .chain()
            .run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnEnter(VisibleGridState::Nest),
        (rerender_ants, rerender_elements, rerender_pheromones)
            .run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnExit(VisibleGridState::Nest),
        (
            despawn_view::<Ant>,
            despawn_view::<Element>,
            despawn_view::<Pheromone>,
        )
            .run_if(in_state(AppState::TellStory)),
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (
            despawn_view::<Ant>,
            cleanup_ants,
            despawn_view::<Element>,
            cleanup_elements,
            despawn_view::<Pheromone>,
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
