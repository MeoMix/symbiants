pub mod ant;
pub mod background;
pub mod element;
pub mod nest_entrance;
pub mod pheromone;

use self::{
    ant::{
        cleanup_ants, on_added_ant_at_crater, on_update_ant_inventory, on_update_ant_orientation,
        on_update_ant_position, spawn_ants,
    },
    background::{cleanup_background, spawn_background, CraterBackground},
    element::{
        cleanup_elements, on_spawn_element, spawn_element_tilemap, spawn_elements, ElementTilemap,
    },
    nest_entrance::{cleanup_nest_entrance, spawn_nest_entrance, NestEntrance},
    pheromone::{on_spawn_pheromone, spawn_pheromones},
};
use crate::common::{
    despawn_view, despawn_view_by_model, on_despawn, on_model_removed_zone, visible_grid::{VisibleGrid, VisibleGridState}
};
use bevy::prelude::*;
use simulation::{
    app_state::AppState,
    common::{ant::Ant, element::Element, pheromone::Pheromone},
    crater_simulation::crater::{AtCrater, Crater},
    CleanupSet,
};

pub struct CraterRenderingPlugin;

impl Plugin for CraterRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Spawn
                (on_spawn_element, on_spawn_pheromone),
                // Despawn
                (
                    on_despawn::<Ant, AtCrater>,
                    on_despawn::<Element, AtCrater>,
                    on_despawn::<Pheromone, AtCrater>,
                ),
                // Added
                (on_added_ant_at_crater),
                // Removed
                (on_model_removed_zone::<AtCrater>),
                // Updated
                (
                    on_update_ant_position,
                    on_update_ant_orientation,
                    on_update_ant_inventory,
                    // on_update_pheromone_visibility,
                ),
            )
                .run_if(
                    in_state(AppState::TellStory)
                        .or_else(in_state(AppState::PostSetupClearChangeDetection)),
                ),
        );

        app.add_systems(
            OnEnter(VisibleGridState::Crater),
            (
                (spawn_element_tilemap),
                apply_deferred,
                (
                    spawn_background,
                    spawn_nest_entrance,
                    spawn_ants,
                    spawn_elements,
                    spawn_pheromones,
                    mark_crater_visible,
                ),
            )
                .chain()
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(VisibleGridState::Crater),
            (
                despawn_view::<CraterBackground>,
                despawn_view::<NestEntrance>,
                despawn_view_by_model::<Ant, AtCrater>,
                despawn_view_by_model::<Element, AtCrater>,
                despawn_view::<ElementTilemap>,
                despawn_view_by_model::<Pheromone, AtCrater>,
                mark_crater_hidden,
            )
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (
                despawn_view::<CraterBackground>,
                cleanup_background,
                despawn_view::<NestEntrance>,
                cleanup_nest_entrance,
                despawn_view_by_model::<Ant, AtCrater>,
                cleanup_ants,
                despawn_view_by_model::<Element, AtCrater>,
                despawn_view::<ElementTilemap>,
                cleanup_elements,
                despawn_view_by_model::<Pheromone, AtCrater>,
            )
                .in_set(CleanupSet::BeforeSimulationCleanup),
        );
    }
}

pub fn mark_crater_visible(
    crater_query: Query<Entity, With<Crater>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(crater_query.single());
}

pub fn mark_crater_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
