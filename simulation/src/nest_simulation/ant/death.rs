use bevy::prelude::*;

use crate::{
    common::{grid::Grid, position::Position},
    nest_simulation::{
        ant::{commands::AntCommandsExt, AntInventory},
        element::Element,
        nest::{AtNest, Nest},
    },
};

use super::Dead;

/// Force ants to drop, or despawn, their inventory upon death.
/// TODO:
///     * It might be preferable to find an adjacent, available zone to move inventory to rather than despawning.
pub fn on_ants_add_dead(
    ants_query: Query<(Entity, &Position, &AntInventory), (Added<Dead>, With<AtNest>)>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    elements_query: Query<&Element>,
) {
    let grid = nest_query.single();

    for (ant_entity, ant_position, ant_inventory) in ants_query.iter() {
        if ant_inventory.0 != None {
            let element_entity = grid.elements().get_element_entity(*ant_position).unwrap();

            if grid
                .elements()
                .is_element(&elements_query, *ant_position, Element::Air)
            {
                commands.drop(ant_entity, *ant_position, *element_entity, AtNest);
            } else {
                commands.entity(*element_entity).remove_parent().despawn();
            }
        }
    }
}