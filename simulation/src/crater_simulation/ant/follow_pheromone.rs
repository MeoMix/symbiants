use bevy::prelude::*;

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

/// Ants will follow Food pheromone if they have no Food in their inventory
/// Ants will follow Nest pheromone if they have Food in their inventory
pub fn ants_follow_pheromone(
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
    grid_elements: GridElements<AtCrater>,
    pheromone_map: Res<PheromoneMap<AtCrater>>,
) {
    for (mut initiative, mut position, mut orientation, inventory) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // TODO: Holy crap this is hardcoded. This could be expressed using math and joining vectors rather than
        // explicitly turning based on known target position.
        let ahead_positions = get_ahead_positions(&orientation, &position, 10);
        let below_positions = get_below_positions(&orientation, &position, 10);
        let above_positions = get_above_positions(&orientation, &position, 10);

        // If ant is carrying food, it should follow the pheromone that leads home.
        // Otherwise, it should follow the pheromone that leads to food.
        // If no pheromones nearby, then just walk randomly.
        // Only consider locations which are walkable (i.e. contain air)
        let search_positions = (ahead_positions
            .iter()
            .chain(below_positions.iter())
            .chain(above_positions.iter()))
        .filter_map(|position| {
            grid_elements.get_entity(*position).and_then(|entity| {
                if *grid_elements.element(*entity) == Element::Air {
                    Some(*position)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

        let desired_pheromone = match inventory.0 {
            Some(_) => Pheromone::Nest,
            None => Pheromone::Food,
        };

        // TODO: Consider reverting this back to the simpler approach of following max strength rather than preferring forward movement.
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
            .max_by(|(_, a), (_, b)| {
                a.value()
                    .partial_cmp(&b.value())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(position, _)| *position);

        if let Some(pheromone_target_position) = pheromone_target_position {
            if ahead_positions.contains(&pheromone_target_position) {
                *position = ahead_positions.first().unwrap().clone();
            } else {
                if below_positions.contains(&pheromone_target_position) {
                    *orientation = orientation.rotate_forward();
                    *position = below_positions.first().unwrap().clone();
                } else if above_positions.contains(&pheromone_target_position) {
                    *orientation = orientation.rotate_backward();
                    *position = above_positions.first().unwrap().clone();
                }
            }

            initiative.consume_movement();
        }
    }
}

fn get_ahead_positions(
    orientation: &AntOrientation,
    start_position: &Position,
    n: usize,
) -> Vec<Position> {
    let mut positions = Vec::new();
    let mut current_position = start_position.clone();

    for _ in 0..n {
        let next_position = orientation.get_ahead_position(&current_position);
        positions.push(next_position.clone());
        current_position = next_position;
    }

    positions
}

fn get_below_positions(
    orientation: &AntOrientation,
    start_position: &Position,
    n: usize,
) -> Vec<Position> {
    let mut positions = Vec::new();
    let mut current_position = start_position.clone();

    for _ in 0..n {
        let next_position = orientation.get_below_position(&current_position);
        positions.push(next_position.clone());
        current_position = next_position;
    }

    positions
}

fn get_above_positions(
    orientation: &AntOrientation,
    start_position: &Position,
    n: usize,
) -> Vec<Position> {
    let mut positions = Vec::new();
    let mut current_position = start_position.clone();

    for _ in 0..n {
        let next_position = orientation.get_above_position(&current_position);
        positions.push(next_position.clone());
        current_position = next_position;
    }

    positions
}
