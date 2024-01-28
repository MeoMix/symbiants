use bevy::prelude::*;

use crate::common::VisibleGrid;

use simulation::nest_simulation::nest::Nest;

pub fn mark_nest_visible(
    nest_query: Query<Entity, With<Nest>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(nest_query.single());
}

pub fn mark_nest_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
