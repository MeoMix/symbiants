use bevy::prelude::*;

use crate::story::{
    ant::{commands::AntCommandsExt, AntInventory},
    common::position::Position,
    element::Element,
    nest_simulation::grid::Grid,
};

use super::Dead;

/// Force ants to drop, or despawn, their inventory upon death.
/// TODO:
///     * It might be preferable to find an adjacent, available location to move inventory to rather than despawning.
pub fn on_ants_add_dead(
    ants_query: Query<(Entity, &Position, &AntInventory), Added<Dead>>,
    mut commands: Commands,
    nest_query: Query<&Grid>,
    elements_query: Query<&Element>,
) {
    let nest = nest_query.single();

    for (ant_entity, ant_position, ant_inventory) in ants_query.iter() {
        if ant_inventory.0 != None {
            let element_entity = nest.elements().get_element_entity(*ant_position).unwrap();

            if nest
                .elements()
                .is_element(&elements_query, *ant_position, Element::Air)
            {
                commands.drop(ant_entity, *ant_position, *element_entity);
            } else {
                commands.entity(*element_entity).remove_parent().despawn();
            }
        }
    }
}
