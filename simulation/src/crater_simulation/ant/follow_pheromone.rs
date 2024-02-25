use std::f32::consts::PI;
use bevy::{prelude::*, utils::HashSet};
use crate::{
    common::{
        ant::{AntInventory, CraterOrientation, Initiative},
        pheromone::{Pheromone, PheromoneMap, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

// TODO: These are both magic. Can adjust them in the future. A smaller field-of-view might help prevent getting stuck
// and going in a circle indefinitely. A smaller distance might be needed for performance at some point.
const FIELD_OF_VIEW: f32 = 180.0;
const DISTANCE: isize = 10;

#[derive(Debug)]
enum Direction {
    Forward,
    Left,
    Right,
}

// TODO: Need to make this logic more robust still. If there's stuff blocking the path between ant and strongest pheromone they'll get stuck.

/// Ants will follow Food pheromone if they have no Food in their inventory
/// Ants will follow Nest pheromone if they have Food in their inventory
pub fn ants_follow_pheromone(
    mut ants_query: Query<
        (
            &mut Initiative,
            &mut Position,
            &mut CraterOrientation,
            &AntInventory,
        ),
        With<AtCrater>,
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength), With<AtCrater>>,
    pheromone_map: Res<PheromoneMap<AtCrater>>,
) {
    for (mut initiative, mut position, mut orientation, inventory) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let positions =
            calculate_positions_in_fov(*position, orientation.as_ref(), FIELD_OF_VIEW, DISTANCE);

        // If ant is carrying food, it should follow the pheromone that leads home.
        // Otherwise, it should follow the pheromone that leads to food.
        let desired_pheromone = match inventory.0 {
            Some(_) => Pheromone::Nest,
            None => Pheromone::Food,
        };

        // Find position of desired pheromone with the highest strength within search positions.
        let pheromone_target_position = positions
            .iter()
            .filter_map(|position| {
                pheromone_map.get(&position).and_then(|pheromone_entities| {
                    pheromone_entities.iter().find_map(|pheromone_entity| {
                        let (pheromone, pheromone_strength) =
                            pheromone_query.get(*pheromone_entity).unwrap();
                        if *pheromone == desired_pheromone {
                            Some((position, pheromone_strength))
                        } else {
                            None
                        }
                    })
                })
            })
            .max_by(|(_, a_strength), (_, b_strength)| {
                a_strength
                    .value()
                    .partial_cmp(&b_strength.value())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(position, _)| *position);

        // TODO: Need to make sure it's possible to walk wherever we're trying to walk - currently can walk through impassable objects
        if let Some(pheromone_target_position) = pheromone_target_position {
            // Calculate direction to pheromone target relative to ant's current orientation
            let direction_to_pheromone =
                calculate_direction_to_target(&position, &orientation, &pheromone_target_position);

            match direction_to_pheromone {
                Direction::Forward => {
                    *position = orientation.get_ahead_position(&position);
                }
                Direction::Left => {
                    let rotated_orientation = orientation.rotate_counterclockwise();
                    *position = rotated_orientation.get_ahead_position(&position);
                    *orientation = rotated_orientation;
                }
                Direction::Right => {
                    let rotated_orientation = orientation.rotate_clockwise();
                    *position = rotated_orientation.get_ahead_position(&position);
                    *orientation = rotated_orientation;
                }
            }

            initiative.consume_movement();
        }
    }
}

/// Note that y-axis is flipped and increases *downward* and trig circle typically
/// represents 0 degrees at 3 o'clock increasing counterclockwise.
/// So, Right maps to 0 degrees and Up maps to 270, not 90, because y-axis is flipped.
fn orientation_to_angle(orientation: &CraterOrientation) -> f32 {
    match orientation {
        CraterOrientation::Up => 270.0,
        CraterOrientation::Right => 0.0,
        CraterOrientation::Down => 90.0,
        CraterOrientation::Left => 180.0,
    }
}

fn calculate_direction_to_target(
    position: &Position,
    orientation: &CraterOrientation,
    target_position: &Position,
) -> Direction {
    // Calculate the vector from the ant to the target
    let vector_to_target = (
        target_position.x - position.x,
        target_position.y - position.y,
    );

    // Convert the ant's orientation into a unit vector
    let orientation_vector = match orientation {
        CraterOrientation::Up => (0, -1),
        CraterOrientation::Right => (1, 0),
        CraterOrientation::Down => (0, 1),
        CraterOrientation::Left => (-1, 0),
    };

    // Calculate the dot product to determine if the target is ahead or behind
    let dot_product =
        orientation_vector.0 * vector_to_target.0 + orientation_vector.1 * vector_to_target.1;

    if dot_product < 0 {
        panic!("Ant is facing away from target - was expecting target to always be within field-of-view");
    }

    // Calculate the determinant (using a 2D cross product concept) to determine left, right, or exactly forward
    let determinant =
        orientation_vector.0 * vector_to_target.1 - orientation_vector.1 * vector_to_target.0;

    if determinant > 0 {
        Direction::Right
    } else if determinant < 0 {
        Direction::Left
    } else {
        Direction::Forward
    }
}

fn calculate_positions_in_fov(
    start_position: Position,
    orientation: &CraterOrientation,
    fov: f32,            // in degrees
    max_distance: isize, // Maximum distance to check from the start position
) -> HashSet<Position> {
    let mut positions = HashSet::new();
    let orientation_angle = orientation_to_angle(orientation);
    let half_fov = fov / 2.0;
    let start_angle = orientation_angle - half_fov;

    for distance in 1..=max_distance {
        let step_angle = calculate_min_step_angle(distance);

        for angle in
            (start_angle as isize..=(start_angle + fov) as isize).step_by(step_angle as usize)
        {
            let rad_angle = angle as f32 * (PI / 180.0);
            let dx = (rad_angle.cos() * distance as f32).round() as isize;
            let dy = (rad_angle.sin() * distance as f32).round() as isize;
            positions.insert(Position {
                x: start_position.x + dx,
                y: start_position.y + dy,
            });
        }
    }

    positions
}

fn calculate_min_step_angle(distance: isize) -> f32 {
    // Assuming tile size is 1 (unit length), and we need half of it for the calculation
    let tile_half_size = 0.5;
    // Calculate the angle in radians
    let step_angle_radians = 2.0 * ((tile_half_size / distance as f32).atan());
    // Convert the angle to degrees
    let step_angle_degrees = step_angle_radians * (180.0 / PI);
    step_angle_degrees
}
