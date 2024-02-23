use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{
        ant::{CraterOrientation, Initiative},
        element::Element,
        grid::GridElements,
        position::Position,
    },
    crater_simulation::crater::AtCrater,
    settings::Settings,
};

/// Ants do a random walk unless they find pheromone relevant to their needs.
/// If they have food then they'll follow Pheromone that leads home.
/// If they have no food then they'll follow Pheromone that leads to food.
pub fn ants_wander(
    mut ants_query: Query<(&mut Initiative, &mut Position, &mut CraterOrientation), With<AtCrater>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    grid_elements: GridElements<AtCrater>,
) {
    for (mut initiative, mut position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(&position);

        let has_air_ahead = grid_elements
            .get_entity(ahead_position)
            .map_or(false, |entity| {
                *grid_elements.element(*entity) == Element::Air
            });

        // An ant might turn randomly. This is to prevent ants from getting stuck in loops, add visual variety, and help it discover food
        let is_turning_randomly = rng.chance(settings.probabilities.random_crater_turn.into());

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
    orientation: &CraterOrientation,
    rng: &mut ResMut<GlobalRng>,
) -> CraterOrientation {
    let all_orientations = CraterOrientation::all_orientations();
    let valid_orientations = all_orientations
        .iter()
        .filter(|&&inner_orientation| inner_orientation != *orientation)
        .collect::<Vec<_>>();

    // TODO: sample?
    *valid_orientations[rng.usize(0..valid_orientations.len())]
}
