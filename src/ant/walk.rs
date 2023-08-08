use crate::{
    element::{is_element, Element},
    map::{Position, WorldMap},
    world_rng::WorldRng, settings::Settings,
};

use super::{AntRole, AntTimer, AntInventory, AntOrientation, Alive};
use bevy::prelude::*;
use rand::Rng;

// Update the position and orientation of all ants. Does not affect the external environment.
pub fn ants_walk(
    mut ants_query: Query<
        (
            &mut Position,
            &mut AntOrientation,
            Ref<AntInventory>,
            &AntTimer,
            &AntRole,
        ),
        With<Alive>,
    >,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
) {
    for (mut position, mut orientation, inventory, timer, role) in ants_query.iter_mut() {
        if timer.0 > 0 {
            continue;
        }

        let below_feet_position = *position + orientation.rotate_forward().get_forward_delta();
        let is_air_beneath_feet = is_element(
            &world_map,
            &elements_query,
            &below_feet_position,
            &Element::Air,
        );

        if is_air_beneath_feet {
            // Whoops, whatever we were walking on disappeared.
            let below_position = *position + Position::Y;
            let is_air_below =
                is_element(&world_map, &elements_query, &below_position, &Element::Air);

            // Gravity system will handle things if going to fall
            if is_air_below {
                continue;
            }

            // Not falling? Try turning
            turn(
                orientation,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
            );

            continue;
        }

        if world_rng.0.gen::<f32>() < settings.probabilities.random_turn {
            turn(
                orientation,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
            );
            continue;
        }

        // Propose taking a step forward, but check validity and alternative actions before stepping forward.
        let new_position = *position + orientation.get_forward_delta();

        if !world_map.is_within_bounds(&new_position) {
            // Hit an edge - need to turn.
            turn(
                orientation,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
            );
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.get_element(new_position).unwrap();
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element != Element::Air {
            // Decided to not dig through and can't walk through, so just turn.
            turn(
                orientation,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
            );
            continue;
        }

        // TODO: Change detection here seems broken
        // HACK: Simulate queen following pheromone back to dig site by forcing her to turn around when dropping dirt on surface.
        if *role == AntRole::Queen
            && inventory.0 == None
            && inventory.is_changed()
            && !world_map.is_below_surface(&new_position)
        {
            info!("HACK QUEEN ANT TURN AROUND");
            turn(
                orientation,
                position,
                &elements_query,
                &mut world_map,
                &mut world_rng,
            );
            continue;
        }

        // Check footing and move forward if possible.
        let foot_orientation = orientation.rotate_forward();
        let foot_position = new_position + foot_orientation.get_forward_delta();

        if let Some(foot_entity) = world_map.get_element(foot_position) {
            let Ok(foot_element) = elements_query.get(*foot_entity) else {
                panic!("act - expected entity to exist")
            };

            if *foot_element == Element::Air {
                // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
                *position = foot_position;
                *orientation = foot_orientation;
            } else {
                // Just move forward
                *position = new_position;
            }
        }
    }
}

fn turn(
    mut orientation: Mut<AntOrientation>,
    position: Mut<Position>,
    elements_query: &Query<&Element>,
    world_map: &mut ResMut<WorldMap>,
    world_rng: &mut ResMut<WorldRng>,
) {
    // First try turning perpendicularly towards the ant's back. If that fails, try turning around.
    let back_orientation = orientation.rotate_backward();
    if is_valid_location(back_orientation, *position, elements_query, world_map) {
        *orientation = back_orientation;
        return;
    }

    let opposite_orientation = orientation.turn_around();
    if is_valid_location(opposite_orientation, *position, elements_query, world_map) {
        *orientation = opposite_orientation;
        return;
    }

    // Randomly turn in a valid different when unable to simply turn around.
    let all_orientations = AntOrientation::all_orientations();
    let valid_orientations = all_orientations
        .iter()
        .filter(|&&inner_orientation| inner_orientation != *orientation)
        .filter(|&&inner_orientation| is_valid_location(inner_orientation, *position, elements_query, world_map))
        .collect::<Vec<_>>();

    if !valid_orientations.is_empty() {
        *orientation = *valid_orientations[world_rng.0.gen_range(0..valid_orientations.len())];
        return;
    }

    info!("TRAPPED");
    *orientation = all_orientations[world_rng.0.gen_range(0..all_orientations.len())];
}


fn is_valid_location(
    orientation: AntOrientation,
    position: Position,
    elements_query: &Query<&Element>,
    world_map: &ResMut<WorldMap>,
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
