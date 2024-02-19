use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{
        ant::{AntInventory, AntOrientation, Initiative},
        element::Element,
        grid::GridElements,
        pheromone::{Pheromone, PheromoneMap, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
    settings::Settings,
};

/// Ants do a random walk unless they find pheromone relevant to their needs.
/// If they have food then they'll follow Pheromone that leads home.
/// If they have no food then they'll follow Pheromone that leads to food.
pub fn ants_walk(
    mut ants_query: Query<
        (
            &mut Initiative,
            &mut Position,
            &mut AntOrientation,
            &AntInventory,
        ),
        With<AtCrater>,
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength), With<AtCrater>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    grid_elements: GridElements<AtCrater>,
    pheromone_map: Res<PheromoneMap<AtCrater>>,
) {
    for (mut initiative, mut position, mut orientation, inventory) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // TODO: Holy crap this is hardcoded. This could be expressed using math and joining vectors rather than
        // explicitly turning based on known target position.
        let ahead_position = orientation.get_ahead_position(&position);
        let below_position = orientation.get_below_position(&position);
        let above_position = orientation.get_above_position(&position);

        let has_air_ahead = grid_elements
            .get_entity(ahead_position)
            .map_or(false, |entity| {
                *grid_elements.element(*entity) == Element::Air
            });

        // If ant is carrying food, it should follow the pheromone that leads home.
        // Otherwise, it should follow the pheromone that leads to food.
        // If no pheromones nearby, then just walk randomly.
        let search_positions = [ahead_position, below_position, above_position];

        let desired_pheromone = match inventory.0 {
            Some(_) => Pheromone::Nest,
            None => Pheromone::Food,
        };

        // Find position of desired pheromone with the highest strength within search positions.
        let pheromone_target_position = search_positions
            .iter()
            .flat_map(|position| {
                pheromone_map
                    .get(position)
                    .iter()
                    .flat_map(|pheromone_entities| {
                        pheromone_entities.iter().filter_map(|pheromone_entity| {
                            let (pheromone, pheromone_strength) =
                                pheromone_query.get(*pheromone_entity).unwrap();
                            if *pheromone == desired_pheromone {
                                Some((position, pheromone_strength))
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .max_by_key(|&(_, pheromone_strength)| pheromone_strength.value())
            .map(|(position, _)| position);

        if let Some(pheromone_target_position) = pheromone_target_position {
            if pheromone_target_position == &ahead_position && has_air_ahead {
                // Just move forward
                *position = ahead_position;
            } else {
                if pheromone_target_position == &below_position {
                    *orientation = orientation.rotate_forward()
                } else if pheromone_target_position == &above_position {
                    *orientation = orientation.rotate_backward()
                }
            }

            initiative.consume_movement();
            continue;
        }

        // An ant might turn randomly. This is to prevent ants from getting stuck in loops and add visual variety.
        let is_turning_randomly = rng.chance(settings.probabilities.random_turn.into());

        if !has_air_ahead || is_turning_randomly {
            *orientation = get_turned_orientation(&orientation, &mut rng);

            initiative.consume_movement();
            continue;
        }

        // Just move forward
        *position = ahead_position;

        initiative.consume_movement();
    }
}

pub fn get_turned_orientation(
    orientation: &AntOrientation,
    rng: &mut ResMut<GlobalRng>,
) -> AntOrientation {
    let all_orientations = AntOrientation::all_orientations();
    let valid_orientations = all_orientations
        .iter()
        .filter(|&&inner_orientation| {
            inner_orientation != *orientation && !inner_orientation.is_upside_down()
        })
        .collect::<Vec<_>>();

    *valid_orientations[rng.usize(0..valid_orientations.len())]
}
