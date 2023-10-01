/// A worker ant may randomly decide to dig a tunnel in a tunnel east/west/south of the nest under the following conditions:
///     1) The ant must not be hungry. If the ant is hungry it's assumed that nest expansion isn't desirable because resources are scarce.
///     2) The ant must feel crowded. If the ant doesn't feel crowded then it's assumed that nest expansion isn't desirable because there's plenty of space.
/// For now, crowding will be a really naive implementation where if an ant has at least two other ants adjacent to it then it is crowded.
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    ant::commands::AntCommandsExt,
    element::Element,
    world_map::{position::Position, WorldMap},
    pheromone::{Pheromone, commands::PheromoneCommandsExt},
    settings::Settings,
};

use super::{hunger::Hunger, AntInventory, AntOrientation, AntRole, Dead, Initiative};

pub fn ants_nest_expansion(
    mut ants_query: Query<
        (
            &AntRole,
            &Hunger,
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
            &Position,
            Entity,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    let ant_entity_positions = ants_query
        .iter_mut()
        .map(|(_, _, _, _, _, position, entity)| (*position, entity))
        .collect::<Vec<_>>();

    for (ant_role, hunger, ant_orientation, inventory, mut initiative, ant_position, ant_entity) in
        ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        if *ant_role != AntRole::Worker
            || inventory.0 != None
            || hunger.is_hungry()
            || world_map.is_aboveground(ant_position)
        {
            continue;
        }

        let is_crowded = ant_entity_positions
            .iter()
            .filter(|(other_ant_position, other_ant_entity)| {
                *other_ant_entity != ant_entity && ant_position.distance(other_ant_position) <= 1
            })
            .count()
            >= 2;

        if is_crowded && rng.f32() < settings.probabilities.expand_nest {
            info!("i want to expand the nest!");

            // Collect a set of positions to check for dirt.
            let mut positions = Vec::new();
            if ant_orientation.is_rightside_up() {
                // If ant is rightside up then it can dig forward (east/west) or below it (south).
                positions.push(ant_orientation.get_ahead_position(ant_position));
                positions.push(ant_orientation.get_below_position(ant_position));
            } else if ant_orientation.is_upside_down() {
                // If ant is upside down then ant can dig forward (east/west) or above it (south).
                positions.push(ant_orientation.get_ahead_position(ant_position));
                positions.push(ant_orientation.get_above_position(ant_position));
            } else {
                // If ant is vertical pointing up then ant can dig above (east/west) or below it (east/west).
                // If ant is vertical pointing down then ant can dig forward (south) above it (east/west) or below it (east/west).
                positions.push(ant_orientation.get_above_position(ant_position));
                positions.push(ant_orientation.get_below_position(ant_position));
                // TODO: double-check logic for facing downward and include digging south
            }

            let dirt_position = positions
                .iter()
                .find(|position| world_map.is_element(&elements_query, **position, Element::Dirt));

            if let Some(dirt_position) = dirt_position {
                // Look for a dirt block to the east/west/south of current position
                // TODO: need to handle the fact an ant could be upside down - I don't want them digging "down" from themselves and going north

                // let dig_position = ant_orientation.get_below_position(ant_position);
                let dig_target_entity = *world_map.element(*dirt_position);
                commands.dig(ant_entity, *dirt_position, dig_target_entity);

                // TODO: maybe consume movement here too since it looks weird when digging down and moving forward in same frame?
                initiative.consume_action();
                // *nesting = Nesting::Started(dig_position);

                commands.spawn_pheromone(*dirt_position, Pheromone::Tunnel);
            }
        }
    }
}
