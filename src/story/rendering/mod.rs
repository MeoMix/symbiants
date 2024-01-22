// TODO: This shouldn't need to be public I think?
pub mod common;
mod crater_rendering;
mod nest_rendering;

use bevy::prelude::*;

use crate::app_state::AppState;

use self::{
    common::{
        despawn_common_entities, initialize_common_resources, on_update_selected,
        on_update_selected_position, remove_common_resources, ModelViewEntityMap,
    },
    crater_rendering::crater::on_crater_removed_visible_grid,
    nest_rendering::{
        ant::{
            ants_sleep_emote, despawn_ants, on_added_ant_dead, on_added_ant_emote, on_ant_ate_food,
            on_ant_wake_up, on_despawn_ant, on_removed_emote, on_spawn_ant, on_tick_emote,
            on_update_ant_color, on_update_ant_inventory, on_update_ant_orientation,
            on_update_ant_position, rerender_ants,
        },
        element::{
            check_element_sprite_sheet_loaded, despawn_elements, on_despawn_element,
            on_update_element, rerender_elements, start_load_element_sprite_sheet,
        },
        nest::{on_nest_removed_visible_grid, on_spawn_nest},
        pheromone::{
            despawn_pheromones, on_despawn_pheromone, on_spawn_pheromone,
            on_update_pheromone_visibility, rerender_pheromones,
        },
    },
};

use super::{
    grid::VisibleGridState,
    story_time::{
        remove_story_time_resources, StoryPlaybackState, StoryTime, DEFAULT_TICKS_PER_SECOND,
    },
};

pub struct RenderingPlugin;

/// Plugin that handles rendering of the simulation. Plugin handles rendering nest and crater views, but only one view will be visible at a time.
/// IMPORTANT:
///     RemovedComponents<T> may contain stale/duplicate data when queried from within Update.
///     This occurs because the simulation may tick multiple times before rendering systems run.
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        build_common_systems(app);
        build_nest_systems(app);
        build_crater_systems(app);
    }
}

fn build_common_systems(app: &mut App) {
    app.add_systems(OnEnter(AppState::FinishSetup), initialize_common_resources);

    app.add_systems(
        Update,
        (on_update_selected, on_update_selected_position)
            // TODO: Unclear if this .chain() is necessary
            .chain()
            .run_if(
                in_state(AppState::TellStory)
                    .and_then(not(in_state(StoryPlaybackState::FastForwarding))),
            ),
    );

    app.add_systems(
        OnExit(AppState::Cleanup),
        |model_view_entity_map: Res<ModelViewEntityMap>| {
            if model_view_entity_map.0.len() > 0 {
                panic!(
                    "ModelViewEntityMap has {} entries remaining after cleanup",
                    model_view_entity_map.0.len()
                );
            }
        },
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (despawn_common_entities, remove_common_resources)
            // TODO: This expectation isn't very clear. The intent is to cleanup rendering before simulation.
            .before(remove_story_time_resources),
    );
}

fn build_crater_systems(app: &mut App) {
    app.add_systems(
        Update,
        (on_crater_removed_visible_grid,).chain().run_if(
            in_state(AppState::TellStory)
                .and_then(not(in_state(StoryPlaybackState::FastForwarding))),
        ),
    );
}

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
            (on_removed_emote, on_nest_removed_visible_grid),
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
            .run_if(
                in_state(AppState::TellStory)
                    .and_then(not(in_state(StoryPlaybackState::FastForwarding))),
            ),
    );

    app.add_systems(
        OnExit(StoryPlaybackState::FastForwarding),
        (rerender_ants, rerender_elements, rerender_pheromones)
            .run_if(in_state(VisibleGridState::Nest)),
    );

    app.add_systems(
        OnEnter(VisibleGridState::Nest),
        (rerender_ants, rerender_elements, rerender_pheromones)
            .run_if(in_state(StoryPlaybackState::Playing)),
    );

    app.add_systems(
        OnEnter(AppState::Cleanup),
        (despawn_ants, despawn_elements, despawn_pheromones)
            // TODO: This expectation isn't very clear. The intent is to cleanup rendering before simulation.
            .before(remove_story_time_resources),
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
