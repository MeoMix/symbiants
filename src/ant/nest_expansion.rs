/// A worker ant may randomly decide to dig a tunnel in a tunnel east/west/south of the nest under the following conditions:
///     1) The ant must not be hungry. If the ant is hungry it's assumed that nest expansion isn't desirable because resources are scarce.
///     2) The ant must feel crowded. If the ant doesn't feel crowded then it's assumed that nest expansion isn't desirable because there's plenty of space.
/// For now, crowding will be a really naive implementation where if an ant has at least two other ants adjacent to it then it is crowded.
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    ant::commands::AntCommandsExt,
    element::Element,
    pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneStrength},
    settings::Settings,
    world_map::{position::Position, WorldMap},
};

use super::{hunger::Hunger, AntInventory, AntOrientation, AntRole, Dead, Initiative};

pub fn ants_nest_expansion(
    ants_query: Query<
        (
            &AntRole,
            &Hunger,
            &AntOrientation,
            &AntInventory,
            &Initiative,
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
        .iter()
        .map(|(_, _, _, _, _, position, entity)| (*position, entity))
        .collect::<Vec<_>>();

    for (ant_role, hunger, ant_orientation, inventory, initiative, ant_position, ant_entity) in
        ants_query.iter()
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
            // Collect a set of positions to check for dirt.
            // These positions are all East/South/West of the ant and are never behind the ant.
            let positions = ant_orientation.get_valid_nonnorth_positions(ant_position);

            let dirt_position = positions
                .iter()
                .find(|position| world_map.is_element(&elements_query, **position, Element::Dirt));

            if let Some(dirt_position) = dirt_position {
                let dig_target_entity = *world_map.element_entity(*dirt_position);
                commands.dig(ant_entity, *dirt_position, dig_target_entity);
                commands.spawn_pheromone(
                    *dirt_position,
                    Pheromone::Tunnel,
                    PheromoneStrength::new(settings.tunnel_length, settings.tunnel_length),
                );
            }
        }
    }
}