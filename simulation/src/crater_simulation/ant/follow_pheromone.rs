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
        let ahead_position = orientation.get_ahead_position(&position);
        let below_position = orientation.get_below_position(&position);
        let above_position = orientation.get_above_position(&position);

        // If ant is carrying food, it should follow the pheromone that leads home.
        // Otherwise, it should follow the pheromone that leads to food.
        // If no pheromones nearby, then just walk randomly.
        // Only consider locations which are walkable (i.e. contain air)
        let search_positions = [ahead_position, below_position, above_position]
        .iter()
        .filter_map(|position| {
            grid_elements
                .get_entity(*position)
                .and_then(|entity| {
                    if *grid_elements.element(*entity) == Element::Air {
                        Some(*position)
                    } else {
                        None
                    }
                })
        }).collect::<Vec<_>>();

        let desired_pheromone = match inventory.0 {
            Some(_) => Pheromone::Nest,
            None => Pheromone::Food,
        };

        // Find position of desired pheromone with the highest strength within search positions.
        let desired_pheromone_positions = search_positions
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
            .collect::<Vec<_>>();

        if desired_pheromone_positions.is_empty() {
            continue;
        }

        let max_strength_value = desired_pheromone_positions
            .iter()
            .map(|(_, strength)| strength.value())
            .max_by_key(|&strength_value| strength_value)
            .unwrap();

        let desired_pheromone_target_positions = desired_pheromone_positions
            .into_iter()
            .filter(|&(_, strength)| strength.value() == max_strength_value)
            .map(|(position, _)| *position) // Dereference to clone the position, if necessary
            .collect::<Vec<_>>();

        // Prefer moving forward if possible - if there's confusion due to a tie in pheromones don't aimlessly spin in a circle.
        if desired_pheromone_target_positions.contains(&ahead_position) {
            *position = ahead_position;
        } else {
            if desired_pheromone_target_positions.contains(&below_position) {
                *orientation = orientation.rotate_forward();
                *position = below_position;
            } else if desired_pheromone_target_positions.contains(&above_position) {
                *orientation = orientation.rotate_backward();
                *position = above_position;
            }
        }

        initiative.consume_movement();
    }
}
