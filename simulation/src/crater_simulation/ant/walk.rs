use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{element::Element, grid::GridElements, position::Position},
    crater_simulation::crater::AtCrater,
    nest_simulation::ant::{AntOrientation, Initiative},
    settings::Settings,
};

/// Ants do a random walk unless they find pheromone relevant to their needs.
/// If they have food then they'll follow Pheromone that leads home.
/// If they have no food then they'll follow Pheromone that leads to food.
pub fn ants_walk(
    mut ants_query: Query<(&mut Initiative, &mut Position, &mut AntOrientation), With<AtCrater>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    grid_elements: GridElements<AtCrater>,
) {
    for (mut initiative, mut position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // TODO: No idea off the top of my head if orientation works for crater like it does for nest, but lets give it a shot.
        let ahead_position = orientation.get_ahead_position(&position);
        let has_air_ahead = grid_elements
            .get_entity(ahead_position)
            .map_or(false, |entity| {
                grid_elements
                    .get_element(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        // An ant might turn randomly. This is to prevent ants from getting stuck in loops and add visual variety.
        let is_turning_randomly = rng.chance(settings.probabilities.random_turn.into());

        if !has_air_ahead || is_turning_randomly {
            *orientation =
                get_turned_orientation(&orientation, &position, &mut rng, &grid_elements);

            initiative.consume_movement();
            continue;
        }

        // Just move forward
        *position = ahead_position;

        initiative.consume_movement();
    }
}

// TODO: Can make it more sophisticated - just turn randomly for now.
pub fn get_turned_orientation(
    orientation: &AntOrientation,
    position: &Position,
    rng: &mut ResMut<GlobalRng>,
    grid_elements: &GridElements<AtCrater>,
) -> AntOrientation {
    let all_orientations = AntOrientation::all_orientations();
    let valid_orientations = all_orientations
        .iter()
        .filter(|&&inner_orientation| inner_orientation != *orientation)
        .filter(|_| is_valid_location(*position, grid_elements))
        .collect::<Vec<_>>();

    if !valid_orientations.is_empty() {
        return *valid_orientations[rng.usize(0..valid_orientations.len())];
    }

    // If no valid orientations, just pick a random orientation.
    all_orientations[rng.usize(0..all_orientations.len())]
}

fn is_valid_location(position: Position, grid_elemenets: &GridElements<AtCrater>) -> bool {
    // Need air at the ants' body for it to be a legal ant zone.
    let Some(entity) = grid_elemenets.get_entity(position) else {
        return false;
    };

    let element = grid_elemenets.element(*entity);
    if *element != Element::Air {
        return false;
    }

    true
}
