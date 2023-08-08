use crate::{
    element::Element,
    map::{Position, WorldMap},
    settings::Settings,
    world_rng::WorldRng,
};

use super::{Dead, AntOrientation, Initiative};
use bevy::prelude::*;
use rand::Rng;

// Update the position and orientation of all ants. Does not affect the external environment.
pub fn ants_walk(
    mut ants_query: Query<(&Initiative, &mut Position, &mut AntOrientation), Without<Dead>>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
) {
    for (initiative, mut position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let under_feet_position = *position + orientation.rotate_forward().get_forward_delta();
        let has_air_under_feet =
            world_map.is_element(&elements_query, under_feet_position, Element::Air);

        let under_body_position = *position + Position::Y;
        let has_air_under_body =
            world_map.is_element(&elements_query, under_body_position, Element::Air);

        // Can't move - falling due to gravity.
        if has_air_under_feet && has_air_under_body {
            continue;
        }

        // Consider turning around instead of walking forward. Necessary when lacking space or firm footing, but also happens randomly.
        let forward_position = *position + orientation.get_forward_delta();
        let has_air_ahead = world_map
            .get_element(forward_position)
            .map_or(false, |entity| {
                elements_query
                    .get(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });
        let is_turning_randomly = world_rng.0.gen::<f32>() < settings.probabilities.random_turn;

        if has_air_under_feet || !has_air_ahead || is_turning_randomly {
            *orientation = get_turned_orientation(
                &orientation,
                &position,
                &elements_query,
                &world_map,
                &mut world_rng,
            );
            continue;
        }

        // Definitely walking forward, but if that results in standing over air then turn on current block.
        let foot_orientation = orientation.rotate_forward();
        let foot_position = forward_position + foot_orientation.get_forward_delta();

        if let Some(foot_entity) = world_map.get_element(foot_position) {
            let foot_element = elements_query.get(*foot_entity).unwrap();

            if *foot_element == Element::Air {
                // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
                *position = foot_position;
                *orientation = foot_orientation;
            } else {
                // Just move forward
                *position = forward_position;
            }
        }
    }
}

fn get_turned_orientation(
    orientation: &AntOrientation,
    position: &Position,
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
) -> AntOrientation {
    // First try turning perpendicularly towards the ant's back. If that fails, try turning around.
    let back_orientation = orientation.rotate_backward();
    if is_valid_location(back_orientation, *position, elements_query, world_map) {
        return back_orientation;
    }

    let opposite_orientation = orientation.turn_around();
    if is_valid_location(opposite_orientation, *position, elements_query, world_map) {
        return opposite_orientation;
    }

    // Randomly turn in a valid different when unable to simply turn around.
    let all_orientations = AntOrientation::all_orientations();
    let valid_orientations = all_orientations
        .iter()
        .filter(|&&inner_orientation| inner_orientation != *orientation)
        .filter(|&&inner_orientation| {
            is_valid_location(inner_orientation, *position, elements_query, world_map)
        })
        .collect::<Vec<_>>();

    if !valid_orientations.is_empty() {
        return *valid_orientations[world_rng.0.gen_range(0..valid_orientations.len())];
    }

    // If no valid orientations, just pick a random orientation.
    all_orientations[world_rng.0.gen_range(0..all_orientations.len())]
}

fn is_valid_location(
    orientation: AntOrientation,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
) -> bool {
    // Need air at the ants' body for it to be a legal ant location.
    let Some(entity) = world_map.get_element(position) else {
        return false;
    };
    let Ok(element) = elements_query.get(*entity) else {
        panic!("is_valid_location - expected entity to exist")
    };

    if *element != Element::Air {
        return false;
    }

    // Get the location beneath the ants' feet and check for air
    let foot_position = position + orientation.rotate_forward().get_forward_delta();
    let Some(entity) = world_map.get_element(foot_position) else {
        return false;
    };
    let Ok(element) = elements_query.get(*entity) else {
        panic!("is_valid_location - expected entity to exist")
    };

    if *element == Element::Air {
        return false;
    }

    true
}
