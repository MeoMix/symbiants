/// A worker ant may randomly decide to dig a tunnel in a tunnel east/west/south of the nest under the following conditions:
///     1) The ant must not be hungry. If the ant is hungry it's assumed that nest expansion isn't desirable because resources are scarce.
///     2) The ant must feel crowded. If the ant doesn't feel crowded then it's assumed that nest expansion isn't desirable because there's plenty of space.
/// For now, crowding will be a really naive implementation where if an ant has at least two other ants adjacent to it then it is crowded.
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    ant::commands::AntCommandsExt,
    common::position::Position,
    element::Element,
    nest_simulation::nest::Nest,
    pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneStrength},
    settings::Settings,
};

use super::{AntInventory, AntOrientation, AntRole, Initiative};

pub fn ants_nest_expansion(
    ants_query: Query<(
        &AntRole,
        &AntOrientation,
        &AntInventory,
        &Initiative,
        &Position,
        Entity,
    )>,
    elements_query: Query<&Element>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
    nest: Res<Nest>,
) {
    let ant_entity_positions = ants_query
        .iter()
        .map(|(_, _, _, _, position, entity)| (*position, entity))
        .collect::<Vec<_>>();

    for (ant_role, ant_orientation, inventory, initiative, ant_position, ant_entity) in
        ants_query.iter()
    {
        if !initiative.can_act() {
            continue;
        }

        if *ant_role != AntRole::Worker
            || inventory.0 != None
            || nest.is_aboveground(ant_position)
            || ant_orientation.is_facing_north()
        {
            continue;
        }

        let is_crowded = ant_entity_positions
            .iter()
            .filter(|(other_ant_position, other_ant_entity)| {
                *other_ant_entity != ant_entity && ant_position.distance(other_ant_position) <= 2
            })
            .count()
            >= 2;

        if is_crowded && rng.f32() < settings.probabilities.expand_nest {
            let dirt_position = ant_orientation.get_ahead_position(ant_position);

            if !nest.is_element(&elements_query, dirt_position, Element::Dirt) {
                continue;
            }

            // Must be attempting to dig a tunnel which means there needs to be dirt on either side of the dig site.
            let dirt_adjacent_position_above = ant_orientation.get_above_position(&dirt_position);
            let dirt_adjacent_position_below = ant_orientation.get_below_position(&dirt_position);
            if nest.is_element(&elements_query, dirt_adjacent_position_above, Element::Air)
                || nest.is_element(&elements_query, dirt_adjacent_position_below, Element::Air)
            {
                continue;
            }

            let dig_target_entity = *nest.element_entity(dirt_position);
            commands.dig(ant_entity, dirt_position, dig_target_entity);
            commands.spawn_pheromone(
                dirt_position,
                Pheromone::Tunnel,
                PheromoneStrength::new(settings.tunnel_length, settings.tunnel_length),
            );
        }
    }
}
