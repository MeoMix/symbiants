use bevy::prelude::*;

use crate::story::{rendering::common::VisibleGrid, simulation::nest_simulation::nest::Nest};

// TODO: It's weird that I have the concept of `VisibleGrid` in addition to `VisibleGridState`
// Generally representing the same state in two different ways is a great way to introduce bugs.
pub fn mark_nest_visible(
    nest_query: Query<Entity, With<Nest>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(nest_query.single());
}

pub fn mark_nest_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
