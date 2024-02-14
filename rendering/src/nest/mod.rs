pub mod ant;
pub mod background;
pub mod element;
pub mod pheromone;

use crate::common::{on_model_removed_zone, visible_grid::set_visible_grid_state_nest};

use self::{
    ant::{
        cleanup_ants, emote::{
            ants_sleep_emote, despawn_expired_emotes, on_added_ant_emote, on_ant_ate_food,
            on_ant_wake_up, on_removed_ant_emote,
        }, on_added_ant_at_nest, on_added_ant_dead, on_update_ant_color, on_update_ant_inventory, on_update_ant_orientation, on_update_ant_position, spawn_ants
    },
    background::{
        cleanup_background, initialize_background_resources, spawn_background,
        spawn_background_tilemap, update_sky_background, Background, BackgroundTilemap,
    },
    element::{
        cleanup_elements, initialize_element_resources, insert_element_exposure_map,
        on_spawn_element, on_update_element_position, process_element_exposure_changed_events,
        remove_element_exposure_map, spawn_element_tilemap, spawn_elements,
        sprite_sheet::{check_element_sprite_sheet_loaded, start_load_element_sprite_sheet},
        update_element_exposure_map, ElementTilemap,
    },
    pheromone::{on_spawn_pheromone, on_update_pheromone_visibility, spawn_pheromones},
};
use super::common::{
    despawn_view, despawn_view_by_model, on_despawn,
    visible_grid::{VisibleGrid, VisibleGridState},
};
use bevy::prelude::*;
use simulation::{
    app_state::AppState,
    common::{ant::Ant, element::Element, pheromone::Pheromone},
    nest_simulation::nest::{AtNest, Nest},
    CleanupSet, FinishSetupSet,
};

pub struct NestRenderingPlugin;

impl Plugin for NestRenderingPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Move these to Common
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
            (
                initialize_background_resources,
                initialize_element_resources,
            )
                .in_set(FinishSetupSet::AfterSimulationFinishSetup),
        );

        app.add_systems(
            Update,
            (
                (
                    update_element_exposure_map,
                    apply_deferred,
                    process_element_exposure_changed_events,
                )
                    .chain(),
                (
                    // Spawn
                    (on_spawn_element, on_spawn_pheromone),
                    // Despawn
                    (
                        on_despawn::<Ant, AtNest>,
                        on_despawn::<Element, AtNest>,
                        on_despawn::<Pheromone, AtNest>,
                    ),
                    // Added
                    (on_added_ant_emote, on_added_ant_dead, on_added_ant_at_nest),
                    // Removed
                    (on_removed_ant_emote, on_model_removed_zone::<AtNest>),
                    // Updated
                    (
                        on_update_ant_position,
                        on_update_ant_orientation,
                        on_update_ant_color,
                        on_update_ant_inventory,
                        on_update_element_position,
                        on_update_pheromone_visibility,
                    ),
                    // Misc
                    (
                        on_ant_ate_food,
                        on_ant_wake_up,
                        // TODO: naming inconsistencies, but probably want to go more this direction rather than away.
                        ants_sleep_emote,
                        despawn_expired_emotes,
                        update_sky_background,
                    ),
                ),
            )
                .chain()
                .run_if(
                    in_state(AppState::TellStory)
                        .or_else(in_state(AppState::PostSetupClearChangeDetection)),
                ),
        );

        // When beginning the story, start by showing the Nest.
        app.add_systems(OnEnter(AppState::TellStory), set_visible_grid_state_nest);

        app.add_systems(
            OnEnter(VisibleGridState::Nest),
            (
                (
                    spawn_background_tilemap,
                    spawn_element_tilemap,
                    insert_element_exposure_map,
                ),
                apply_deferred,
                (
                    spawn_background,
                    spawn_ants,
                    spawn_elements,
                    spawn_pheromones,
                    mark_nest_visible,
                ),
            )
                .chain()
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(VisibleGridState::Nest),
            (
                despawn_view::<Background>,
                despawn_view::<BackgroundTilemap>,
                despawn_view_by_model::<Ant, AtNest>,
                despawn_view_by_model::<Element, AtNest>,
                despawn_view::<ElementTilemap>,
                despawn_view_by_model::<Pheromone, AtNest>,
                remove_element_exposure_map,
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
                despawn_view_by_model::<Ant, AtNest>,
                cleanup_ants,
                despawn_view_by_model::<Element, AtNest>,
                despawn_view::<ElementTilemap>,
                cleanup_elements,
                despawn_view_by_model::<Pheromone, AtNest>,
            )
                .in_set(CleanupSet::BeforeSimulationCleanup),
        );
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
