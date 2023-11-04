use bevy::prelude::*;

use super::VisibleGrid;

// TODO: Prefer attaching the SpatialBundle here, or having it be an entirely separate entity.
// pub fn on_spawn_grid(grid_query: Query<Entity, (Added<Grid>, Option<VisibleGrid>)>, mut commands: Commands) {
//     for grid_entity in grid_query.iter() {
//         commands.entity(grid_entity).insert(Visibility::Hidden);
//     }
// }

pub fn on_added_visible_grid(
    grid_query: Query<Entity, Added<VisibleGrid>>,
    mut commands: Commands,
) {
    for grid_entity in grid_query.iter() {
        commands.entity(grid_entity).insert(Visibility::Visible);
    }
}

pub fn on_removed_visible_grid(
    mut removed: RemovedComponents<VisibleGrid>,
    mut commands: Commands,
) {
    for grid_entity in &mut removed {
        commands.entity(grid_entity).insert(Visibility::Hidden);
    }
}
