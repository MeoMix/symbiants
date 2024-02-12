use crate::common::{
    ant::{commands::AntCommandsExt, AntInventory, Dead},
    element::Element,
    grid::GridElements,
    position::Position,
    Zone,
};
use bevy::prelude::*;

/// Force ants to drop, or despawn, their inventory upon death.
/// TODO:
///     * It might be preferable to find an adjacent, available zone to move inventory to rather than despawning.
pub fn on_ants_add_dead<Z: Zone + Copy>(
    ants_query: Query<(Entity, &Position, &AntInventory, &Z), (Added<Dead>, With<Z>)>,
    mut commands: Commands,
    grid_elements: GridElements<Z>,
) {
    for (ant_entity, ant_position, ant_inventory, zone) in ants_query.iter() {
        if ant_inventory.0 != None {
            let element_entity = grid_elements.entity(*ant_position);

            if grid_elements.is(*ant_position, Element::Air) {
                commands.drop(ant_entity, *ant_position, *element_entity, *zone);
            } else {
                commands.entity(*element_entity).remove_parent().despawn();
            }
        }
    }
}
