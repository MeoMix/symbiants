use bevy::prelude::*;

use crate::story::crater_simulation::crater::{AtCrater, Crater};

use super::common::{ModelViewEntityMap, VisibleGrid};

pub fn on_added_crater_visible_grid(
    crater_query: Query<&Crater>,
    visible_grid: Res<VisibleGrid>,
    mut at_crater_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtCrater>,
    >,
) {
    if !visible_grid.is_changed() {
        return;
    }

    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if !crater_query.get(visible_grid_entity).is_ok() {
        return;
    }

    for (tile_visible, visibility) in at_crater_view_query.iter_mut() {
        if let Some(mut tile_visibile) = tile_visible {
            tile_visibile.0 = true;
        } else if let Some(mut visibility) = visibility {
            *visibility = Visibility::Visible;
        }
    }
}

// TODO: sheesh this entire method sucks.
// TODO: naming of this vs added doesn't have parity
pub fn on_crater_removed_visible_grid(
    visible_grid: Res<VisibleGrid>,
    crater_query: Query<&Crater>,
    at_crater_view_query: Query<
        Entity,
        // TODO: Better way to select just views? Should I introduce a View component?
        (With<AtCrater>, Or<(With<TileVisible>, With<Visibility>)>),
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
        if crater_query.get(visible_grid_entity).is_ok() {
            return;
        }
    }

    for entity in at_crater_view_query.iter() {
        commands.entity(entity).despawn_recursive();
        // TODO: maybe I need to update my tilemap storage etc when despawning here?
        // tile_storage.set(&tile_pos, element_view_entity);
    }

    // TODO: instead of clearing -- maybe try to carefully remove? should be safe to clear though since switching major views?
    model_view_entity_map.0.clear();
}
