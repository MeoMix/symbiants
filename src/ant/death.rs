use bevy::prelude::*;

use crate::{
    ant::{commands::AntCommandsExt, AntInventory, Initiative},
    element::Element,
    world_map::{position::Position, WorldMap},
};

use super::Dead;

pub fn on_ants_add_dead(
    ants_query: Query<(Entity, &Position, &AntInventory, &Initiative), Added<Dead>>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    elements_query: Query<&Element>,
) {
    for (ant_entity, ant_position, ant_inventory, ant_initiative) in ants_query.iter() {
        // If the ant is carrying something - drop it first.
        if ant_inventory.0 != None {
            // If the ant is standing on air then drop element where standing otherwise despawn element.
            // TODO: in the future maybe try to find an adjacent place to drop element.
            // TODO: it's kind of weird this is element_entity not inventory_item
            let element_entity = world_map.get_element_entity(*ant_position).unwrap();

            if world_map.is_element(&elements_query, *ant_position, Element::Air) {
                // TODO: This is kind of d weird. `commands.drop` demands we have initative. The idea a dead ant has initative
                // is weird, but also if a dead ant is falling through the sky then gravity will have stolen its initative.
                // This means there's a bug in this code where an ant that dies in a position where it can fall, and is also holding something,
                // will despawn its inventory rather than dropping it while falling.
                if ant_initiative.can_act() {
                    commands.drop(ant_entity, *ant_position, *element_entity);
                } else {
                    commands.entity(*element_entity).remove_parent().despawn();
                }
            } else {
                // No room - despawn inventory.
                commands.entity(*element_entity).remove_parent().despawn();
            }
        }
    }
}
