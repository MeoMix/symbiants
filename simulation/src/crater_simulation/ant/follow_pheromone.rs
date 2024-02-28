use crate::{
    common::{
        ant::{AntInventory, AntName, CraterOrientation, Initiative},
        element::Element,
        grid::GridElements,
        pheromone::{Pheromone, PheromoneMap, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

const DETECTION_DISTANCE: f32 = 1.5;

#[derive(Debug)]
enum Direction {
    Forward,
    Left,
    Right,
}

// TODO: Need to make this logic more robust still. If there's stuff blocking the path between ant and strongest pheromone they'll get stuck.
// At time of writing, though, there's nothing else blocking between food and ants aside from the nest entrance which isn't a huge deal.
pub fn ants_follow_pheromone(
    mut ants_query: Query<
        (
            &mut Initiative,
            &mut Position,
            &mut CraterOrientation,
            &AntInventory,
            &AntName,
        ),
        With<AtCrater>,
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength), With<AtCrater>>,
    pheromone_map: Res<PheromoneMap<AtCrater>>,
    grid_elements: GridElements<AtCrater>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut initiative, mut position, mut orientation, inventory, ant_name) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // Don't always follow pheromone to prevent getting stuck in small loops when there's not enough of a trail
        if !rng.chance(settings.probabilities.crater_follow_pheromone.into()) {
            continue;
        }

        let positions = calculate_positions_in_halfcircle(*position, DETECTION_DISTANCE as f32, orientation.as_ref());

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

        if let Some(pheromone_target_position) = pheromone_target_position {
            // Calculate direction to pheromone target relative to ant's current orientation
            let direction_to_pheromone =
                calculate_direction_to_target(&position, &orientation, &pheromone_target_position, ant_name);

            let (new_position, new_orientation) = match direction_to_pheromone {
                Direction::Forward => (orientation.get_ahead_position(&position), None),
                Direction::Left => {
                    let rotated_orientation = orientation.rotate_counterclockwise();
                    (
                        rotated_orientation.get_ahead_position(&position),
                        Some(rotated_orientation),
                    )
                }
                Direction::Right => {
                    let rotated_orientation = orientation.rotate_clockwise();
                    (
                        rotated_orientation.get_ahead_position(&position),
                        Some(rotated_orientation),
                    )
                }
            };

            // Check to ensure can walk to desired position - if not then do nothing and rely on wandering for movement.
            let is_air_at_new_position = grid_elements
                .get_entity(new_position)
                .map_or(false, |entity| {
                    *grid_elements.element(*entity) == Element::Air
                });

            if !is_air_at_new_position {
                continue;
            }

            *position = new_position;

            if let Some(new_orientation) = new_orientation {
                *orientation = new_orientation;
            }

            initiative.consume_movement();
        }
    }
}

fn calculate_direction_to_target(
    position: &Position,
    orientation: &CraterOrientation,
    target_position: &Position,
    ant_name: &AntName,
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
        panic!("Ant {:?} is facing away from target - was expecting target to always be within field-of-view. orientation: {:?}, dot_product: {:?}, position: {:?}, target_position: {:?}", ant_name, orientation, dot_product, position, target_position);
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

/// This relies on an implicit field-of-view of 180 degrees.
/// Its implementation is inspired by https://www.redblobgames.com/grids/circle-drawing/
/// If dynamic field-of-view is required in the future then will need to introduce `atan2` to filter on degrees. 
fn calculate_positions_in_halfcircle(center: Position, radius: f32, orientation: &CraterOrientation) -> Vec<Position> {
    let mut positions = vec![];

    // Calculate the bounding box of the circle to determine which tile positions to check.
    let top = (center.y as f32 - radius).ceil() as isize;
    let bottom = (center.y as f32 + radius).floor() as isize;
    let left = (center.x as f32 - radius).ceil() as isize;
    let right = (center.x as f32 + radius).floor() as isize;

    // Adjust the bounding box based on the direction for a half-circle
    let (adjusted_top, adjusted_bottom, adjusted_left, adjusted_right) = match orientation {
        CraterOrientation::Up => (top, center.y, left, right),
        CraterOrientation::Down => (center.y, bottom, left, right),
        CraterOrientation::Right => (top, bottom, center.x, right),
        CraterOrientation::Left => (top, bottom, left, center.x),
    };

    for y in adjusted_top..adjusted_bottom + 1 {
        for x in adjusted_left..adjusted_right + 1 {
            let dx = center.x - x;
            let dy = center.y - y;
            let distance_squared = dx * dx + dy * dy;
            
            if distance_squared > 0 && distance_squared as f32 <= radius * radius {
                positions.push(Position::new(x, y));
            }
        }
    }

    positions
}