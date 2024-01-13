use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileVisible;

use crate::story::common::ui::{ModelViewEntityMap, VisibleGrid};

use super::{AtCrater, Crater};

pub fn on_added_at_crater(
    crater_query: Query<&Crater>,
    visible_grid: Res<VisibleGrid>,
    mut at_crater_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        Added<AtCrater>,
    >,
) {
    let mut is_crater_visible = false;
    let mut crater_visibility = Visibility::Hidden;

    if let Some(visible_grid_entity) = visible_grid.0 {
        if crater_query.get(visible_grid_entity).is_ok() {
            crater_visibility = Visibility::Visible;
            is_crater_visible = true;
        }
    }

    for (tile_visible, visibility) in at_crater_view_query.iter_mut() {
        // TODO: It's lame that this needs to be aware of two different concepts of visiblity - self-managed vs bevy_ecs_tilemap
        if let Some(mut tile_visibile) = tile_visible {
            tile_visibile.0 = is_crater_visible;
        } else if let Some(mut visibility) = visibility {
            *visibility = crater_visibility;
        }
    }
}

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

// TODO: naming of this vs added doesn't have parity
pub fn on_crater_removed_visible_grid(
    visible_grid: Res<VisibleGrid>,
    crater_query: Query<&Crater>,
    mut at_crater_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtCrater>,
    >,
) {
    if !visible_grid.is_changed() {
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

    for (tile_visible, visibility) in at_crater_view_query.iter_mut() {
        if let Some(mut tile_visible) = tile_visible {
            tile_visible.0 = false;
        } else if let Some(mut visibility) = visibility {
            *visibility = Visibility::Hidden;
        }
    }
}
