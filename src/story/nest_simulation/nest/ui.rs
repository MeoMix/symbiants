use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileVisible;

use crate::story::common::ui::VisibleGrid;

use super::{AtNest, Nest};

// Assume for now that when the simulation loads the user wants to see their nest, but in the future might need to make this more flexible.
pub fn on_spawn_nest(
    nest_query: Query<Entity, Added<Nest>>,
    mut commands: Commands,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    let nest_entity = match nest_query.get_single() {
        Ok(nest_entity) => nest_entity,
        Err(_) => return,
    };

    visible_grid.0 = Some(nest_entity);
}

pub fn on_added_at_nest(
    nest_query: Query<&Nest>,
    visible_grid: Res<VisibleGrid>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        Added<AtNest>,
    >,
) {
    let mut is_nest_visible = false;
    let mut nest_visibility = Visibility::Hidden;

    if let Some(visible_grid_entity) = visible_grid.0 {
        if nest_query.get(visible_grid_entity).is_ok() {
            nest_visibility = Visibility::Visible;
            is_nest_visible = true;
        }
    }

    for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
        // TODO: It's lame that this needs to be aware of two different concepts of visiblity - self-managed vs bevy_ecs_tilemap
        if let Some(mut tile_visibile) = tile_visible {
            tile_visibile.0 = is_nest_visible;
        } else if let Some(mut visibility) = visibility {
            *visibility = nest_visibility;
        }
    }
}

pub fn on_added_nest_visible_grid(
    nest_query: Query<&Nest>,
    visible_grid: Res<VisibleGrid>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtNest>,
    >,
) {
    if !visible_grid.is_changed() {
        return;
    }

    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if !nest_query.get(visible_grid_entity).is_ok() {
        return;
    }

    for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
        if let Some(mut tile_visibile) = tile_visible {
            tile_visibile.0 = true;
        } else if let Some(mut visibility) = visibility {
            *visibility = Visibility::Visible;
        }
    }
}

// TODO: naming of this vs added doesn't have parity
pub fn on_nest_removed_visible_grid(
    visible_grid: Res<VisibleGrid>,
    nest_query: Query<&Nest>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtNest>,
    >,
) {
    if !visible_grid.is_changed() {
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

    for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
        if let Some(mut tile_visible) = tile_visible {
            tile_visible.0 = false;
        } else if let Some(mut visibility) = visibility {
            *visibility = Visibility::Hidden;
        }
    }
}
