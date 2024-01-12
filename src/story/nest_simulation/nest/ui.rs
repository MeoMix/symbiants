use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileVisible;

use crate::story::grid::VisibleGrid;

use super::{AtNest, Nest};

// Assume for now that when the simulation loads the user wants to see their nest, but in the future might need to make this more flexible.
pub fn on_spawn_nest(nest_query: Query<Entity, Added<Nest>>, mut commands: Commands) {
    for nest_entity in nest_query.iter() {
        // TODO: Stop attaching VisibleGrid to Nest - it's a view concern not a model concern.
        commands.entity(nest_entity).insert(VisibleGrid);
    }
}

pub fn on_added_at_nest(
    nest_query: Query<&Nest, With<VisibleGrid>>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        Added<AtNest>,
    >,
) {
    let is_nest_visible = nest_query.get_single().is_ok();
    let nest_visibility = if is_nest_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
        if let Some(mut tile_visibile) = tile_visible {
            tile_visibile.0 = is_nest_visible;
        } else if let Some(mut visibility) = visibility {
            *visibility = nest_visibility;
        }
    }
}

pub fn on_added_nest_visible_grid(
    nest_query: Query<&Nest, Added<VisibleGrid>>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtNest>,
    >,
) {
    if nest_query.get_single().is_ok() {
        for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
            if let Some(mut tile_visibile) = tile_visible {
                tile_visibile.0 = true;
            } else if let Some(mut visibility) = visibility {
                *visibility = Visibility::Visible;
            }
        }
    }
}

// TODO: naming of this vs added doesn't have parity
pub fn on_nest_removed_visible_grid(
    mut removed: RemovedComponents<VisibleGrid>,
    nest_query: Query<&Nest>,
    mut at_nest_view_query: Query<
        (Option<&mut TileVisible>, Option<&mut Visibility>),
        With<AtNest>,
    >,
) {
    for entity in &mut removed.read() {
        // If Nest was the one who had VisibleGrid removed
        if let Ok(_) = nest_query.get(entity) {
            for (tile_visible, visibility) in at_nest_view_query.iter_mut() {
                if let Some(mut tile_visible) = tile_visible {
                    tile_visible.0 = false;
                } else if let Some(mut visibility) = visibility {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
