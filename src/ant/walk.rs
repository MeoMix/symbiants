use crate::{
    element::Element,
    grid::{position::Position, WorldMap},
    settings::Settings,
};

use super::{AntOrientation, Dead, Initiative};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

/// Ants walking vertically may have the ground beneath their feet disappear.
/// Usually, this means gravity will take over and they'll fall downward, but sometimes they have a stable
/// element to the south of them. In that case, they'll rotate to regain their footing.
/// TODO:
///     * I think there's a bug where ant standing adjacent to the edge of the world map will not fall because it's not air.
///     * Code could be written better such that movement initiative isn't consumed if there's not a valid orientation to turn to.
pub fn ants_stabilize_footing_movement(
    mut ants_query: Query<(&mut Initiative, &Position, &mut AntOrientation), Without<Dead>>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut initiative, position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let below_position = orientation.get_below_position(&position);
        let has_air_below = world_map.is_element(&elements_query, below_position, Element::Air);
        if !has_air_below {
            continue;
        }

        *orientation = get_turned_orientation(
            &orientation,
            &position,
            &elements_query,
            &world_map,
            &mut rng,
        );

        info!("ants_stabilize_footing_movement - consumed movement");
        initiative.consume_movement();
    }
}

// Update the position and orientation of all ants. Does not affect the external environment.
pub fn ants_walk(
    mut ants_query: Query<(&mut Initiative, &mut Position, &mut AntOrientation), Without<Dead>>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut initiative, mut position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            info!("ants_walk - initiative cannot move");
            continue;
        }

        info!("ants_walk - can move");

        // An ant might be attempting to walk forward into a solid block. If so, they'll turn and walk up the block.
        let ahead_position = orientation.get_ahead_position(&position);
        let has_air_ahead = world_map
            .get_element(ahead_position)
            .map_or(false, |entity| {
                elements_query
                    .get(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        // An ant might turn randomly. This is to prevent ants from getting stuck in loops and add visual variety.
        let is_turning_randomly = rng.chance(settings.probabilities.random_turn.into());

        if !has_air_ahead || is_turning_randomly {
            *orientation = get_turned_orientation(
                &orientation,
                &position,
                &elements_query,
                &world_map,
                &mut rng,
            );

            initiative.consume_movement();
            continue;
        }

        // Definitely walking forward, but if that results in standing over air then turn on current block.
        let foot_orientation = orientation.rotate_forward();
        let foot_position = foot_orientation.get_ahead_position(&ahead_position);

        if let Some(foot_entity) = world_map.get_element(foot_position) {
            let foot_element = elements_query.get(*foot_entity).unwrap();

            if *foot_element == Element::Air {
                // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
                *position = foot_position;
                *orientation = foot_orientation;
            } else {
                // Just move forward
                *position = ahead_position;
            }

            initiative.consume_movement();
        }
    }
}

// TODO: coupling..
pub fn get_turned_orientation(
    orientation: &AntOrientation,
    position: &Position,
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
    rng: &mut ResMut<GlobalRng>,
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
        return *valid_orientations[rng.usize(0..valid_orientations.len())];
    }

    // If no valid orientations, just pick a random orientation.
    all_orientations[rng.usize(0..all_orientations.len())]
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
    let below_position = orientation.get_below_position(&position);
    let Some(entity) = world_map.get_element(below_position) else {
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
