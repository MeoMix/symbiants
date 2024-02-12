use crate::{
    common::{
        ant::{commands::AntCommandsExt, AntInventory, Dead},
        element::Element,
        grid::GridElements,
        position::Position,
    },
    nest_simulation::nest::AtNest,
};
use bevy::prelude::*;

/// Force ants to drop, or despawn, their inventory upon death.
/// TODO:
///     * It might be preferable to find an adjacent, available zone to move inventory to rather than despawning.
pub fn on_ants_add_dead(
    ants_query: Query<(Entity, &Position, &AntInventory), (Added<Dead>, With<AtNest>)>,
    mut commands: Commands,
    grid_elements: GridElements<AtNest>,
) {
    for (ant_entity, ant_position, ant_inventory) in ants_query.iter() {
        if ant_inventory.0 != None {
            let element_entity = grid_elements.entity(*ant_position);

            if grid_elements.is(*ant_position, Element::Air) {
                commands.drop(ant_entity, *ant_position, *element_entity, AtNest);
            } else {
                commands.entity(*element_entity).remove_parent().despawn();
            }
        }
    }
}
