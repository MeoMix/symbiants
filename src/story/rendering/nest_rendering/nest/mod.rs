use bevy::prelude::*;

use crate::story::{rendering::common::VisibleGrid, simulation::nest_simulation::nest::Nest};

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
