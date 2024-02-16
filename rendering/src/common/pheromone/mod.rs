use bevy::prelude::*;
use simulation::common::{grid::Grid, pheromone::Pheromone};

use super::{visible_grid::VisibleGrid, ModelViewEntityMap};

#[derive(Resource)]
pub struct PheromoneVisibility(pub Visibility);

pub fn initialize_pheromone_resources(mut commands: Commands) {
    commands.insert_resource(PheromoneVisibility(Visibility::Visible));
}

/// Remove resources, etc.
pub fn cleanup_pheromones(mut commands: Commands) {
    commands.remove_resource::<PheromoneVisibility>();
}

// TODO: Instead of toggling visibility, I think it would make more sense to despawn/spawn the pheromone renderings.
// This would reduce the number of entities in the world and improve code reuse.
pub fn on_update_pheromone_visibility(
    pheromone_model_query: Query<Entity, With<Pheromone>>,
    mut pheromone_view_query: Query<&mut Visibility>,
    pheromone_visibility: Res<PheromoneVisibility>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    grid_query: Query<&Grid>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    if !pheromone_visibility.is_changed() {
        return;
    }

    for pheromone_model_entity in pheromone_model_query.iter() {
        if let Some(pheromone_view_entity) = model_view_entity_map.get(&pheromone_model_entity) {
            if let Ok(mut visibility) = pheromone_view_query.get_mut(*pheromone_view_entity) {
                *visibility = pheromone_visibility.0;
            }
        }
    }
}
