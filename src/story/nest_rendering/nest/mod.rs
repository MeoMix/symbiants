use bevy::{prelude::*, render::view::visibility};

use crate::story::nest_simulation::nest::{AtNest, Nest};

use super::common::{ModelViewEntityMap, VisibleGrid};

// Assume for now that when the simulation loads the user wants to see their nest, but in the future might need to make this more flexible.
pub fn on_spawn_nest(
    nest_query: Query<Entity, Added<Nest>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    let nest_entity = match nest_query.get_single() {
        Ok(nest_entity) => nest_entity,
        Err(_) => return,
    };

    visible_grid.0 = Some(nest_entity);
}

// TODO: sheesh this entire method sucks.
// TODO: naming of this vs added doesn't have parity
pub fn on_nest_removed_visible_grid(
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Nest>,
    at_nest_view_query: Query<
        Entity,
        // TODO: Better way to select just views? Should I introduce a View component?
        (With<AtNest>, Or<(With<TileVisible>, With<Visibility>)>),
    >,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    // is_changed() is true when is_added() is true, trying to detect changed as in "changed from one value to another"
    if !visible_grid.is_changed() || visible_grid.is_added() {
        return;
    }

    // TODO: This is somewhat hacky, but IDK if there's much to be done about it.
    // The issue is that if there were 3+ potential visible grids then this would run more often than desired
    // because it would try to remove visibility from nest elements when changing visibility between two non-nest grids.
    if let Some(visible_grid_entity) = visible_grid.0 {
        if nest_query.get(visible_grid_entity).is_ok() {
            return;
        }
    }

    for entity in at_nest_view_query.iter() {
        commands.entity(entity).despawn_recursive();
        // TODO: maybe I need to update my tilemap storage etc when despawning here?
        // tile_storage.set(&tile_pos, element_view_entity);
    }

    // TODO: instead of clearing -- maybe try to carefully remove? should be safe to clear though since switching major views?
    model_view_entity_map.0.clear();
}
