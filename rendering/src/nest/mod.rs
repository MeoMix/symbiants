pub mod ant;
pub mod background;
pub mod element;
pub mod pheromone;

use self::{
    ant::{
        cleanup_ants,
        emote::{
            ants_sleep_emote, despawn_expired_emotes, on_added_ant_emote, on_ant_ate_food,
            on_ant_wake_up, on_removed_ant_emote,
        },
        on_added_ant_dead, on_spawn_ant, on_update_ant_color, on_update_ant_inventory,
        on_update_ant_orientation, on_update_ant_position, rerender_ants,
    },
    background::{
        cleanup_background, spawn_background, spawn_background_tilemap, update_sky_background,
        Background, BackgroundTilemap,
    },
    element::{
        cleanup_elements, on_spawn_element, on_update_element_exposure, on_update_element_position,
        rerender_elements,
        sprite_sheet::{
            check_element_sprite_sheet_loaded, start_load_element_sprite_sheet, ElementTilemap,
        },
    },
    pheromone::{
        cleanup_pheromones, on_spawn_pheromone, on_update_pheromone_visibility, rerender_pheromones,
    },
};
use super::common::{
    despawn_view, despawn_view_by_model, on_despawn,
    visible_grid::{VisibleGrid, VisibleGridState},
};
use bevy::prelude::*;
use simulation::{
    app_state::AppState,
    nest_simulation::{
        ant::Ant,
        element::Element,
        nest::{AtNest, Nest},
        pheromone::Pheromone,
    },
    story_time::{StoryTime, DEFAULT_TICKS_PER_SECOND},
    CleanupSet, FinishSetupSet,
};

pub struct NestRenderingPlugin;

impl Plugin for NestRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::BeginSetup),
            start_load_element_sprite_sheet,
        );

        app.add_systems(
            Update,
            check_element_sprite_sheet_loaded.run_if(in_state(AppState::BeginSetup)),
        );

        // TODO: It's unfortunate this is necessary. It would be nice to drive this all via `OnEnter(VisibleGridState:Nest)`
        // Also, it naively assumes that Nest is rendered on first load. This is true but isn't an assumption that should be made.
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
                (on_added_ant_emote, on_added_ant_dead),
                // Removed
                (on_removed_ant_emote),
                // Updated
                (
                    on_update_ant_position,
                    on_update_ant_orientation,
                    on_update_ant_color,
                    on_update_ant_inventory,
                    on_update_element_position,
                    on_update_element_exposure,
                    on_update_pheromone_visibility,
                ),
                // Misc
                (
                    on_ant_ate_food,
                    on_ant_wake_up,
                    // TODO: naming inconsistencies, but probably want to go more this direction rather than away.
                    ants_sleep_emote.run_if(
                        // TODO: this feels hacky? trying to rate limit how often checks for sleeping emoting occurs.
                        resource_exists::<StoryTime>()
                            .and_then(tick_count_elapsed(DEFAULT_TICKS_PER_SECOND)),
                    ),
                    despawn_expired_emotes,
                ),
            )
                .run_if(in_state(AppState::TellStory)),
        );

        // When beginning the story, start by showing the Nest.
        app.add_systems(OnEnter(AppState::TellStory), mark_nest_visible);

        app.add_systems(
            // Note that the run condition below prevents these systems from running on app load.
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

pub fn mark_nest_visible(
    nest_query: Query<Entity, With<Nest>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(nest_query.single());
}

pub fn mark_nest_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
