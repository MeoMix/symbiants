use bevy::prelude::*;

use super::{elements::Element, Position, WorldMap};
use rand::Rng;

// Returns true if every element in `positions` is Element::Air
// NOTE: This returns true if given 0 positions.
fn is_all_air(
    world_map: &WorldMap,
    elements_query: &Query<(&Element, &mut Position)>,
    positions: Vec<Position>,
) -> bool {
    positions
        .iter()
        .map(|position| {
            let Some(&element) = world_map.elements.get(&position) else { return false; };
            let Ok((&element, _)) = elements_query.get(element) else { return false; };
            element == Element::Air
        })
        .all(|is_air| is_air)
}

// PERF: could introduce 'active' concept and not consider all elements all the time
// TODO: add sand crushing to dirt
// For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
// For each sand element which is above air, swap it with the air beneath it.
fn sand_gravity_system(
    mut elements_query: Query<(&Element, &mut Position)>,
    mut world_map: ResMut<WorldMap>,
) {
    let swaps: Vec<_> = elements_query
        .iter()
        .filter(|(&element, _)| element == Element::Sand)
        .filter_map(|(_, &sand_position)| {
            let mut go_left = false;
            let mut go_right = false;

            let below_sand_position = sand_position + Position::Y;
            let left_sand_position = sand_position + Position::NEG_X;
            let left_below_sand_position = sand_position + Position::new(-1, 1);
            let right_sand_position = sand_position + Position::X;
            let right_below_sand_position = sand_position + Position::ONE;

            // If there is air below the sand then continue falling down.
            let go_down = is_all_air(&world_map, &elements_query, vec![below_sand_position]);

            // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
            // Look for a column of air two units tall to either side of the sand and consider going in one of those directions.
            if !go_down {
                go_left = is_all_air(
                    &world_map,
                    &elements_query,
                    vec![left_sand_position, left_below_sand_position],
                );

                go_right = is_all_air(
                    &world_map,
                    &elements_query,
                    vec![right_sand_position, right_below_sand_position],
                );

                // Flip a coin and choose a direction randomly to resolve ambiguity in fall direction.
                if go_left && go_right {
                    // TODO: control rand seed more for reliable testing
                    if rand::thread_rng().gen_range(0..10) < 5 {
                        go_left = false;
                    } else {
                        go_right = false;
                    }
                }
            }

            let target_position = if go_down {
                Some(below_sand_position)
            } else if go_left {
                Some(left_below_sand_position)
            } else if go_right {
                Some(right_below_sand_position)
            } else {
                None
            };

            target_position.and_then(|target_position| {
                let &air_entity = world_map.elements.get(&target_position)?;
                let &sand_entity = world_map.elements.get(&sand_position)?;
                Some((air_entity, sand_entity))
            })
        })
        .collect();

    for &(air_entity, sand_entity) in swaps.iter() {
        let Ok([(_, mut air_position), (_, mut sand_position)]) = elements_query.get_many_mut([air_entity, sand_entity]) else { continue };

        // Swap element positions internally.
        (sand_position.x, air_position.x) = (air_position.x, sand_position.x);
        (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

        // TODO: maybe use references to position instead?
        // Update indices since they're indexed by position and track where elements are at.
        world_map.elements.insert(*sand_position, sand_entity);
        world_map.elements.insert(*air_position, air_entity);
    }
}

// TODO: Figure out headless testing (prefer over node) and how to run single test
// TODO: It would be much nicer to have a spec to create world state from rather than manually defining it
#[cfg(test)]
pub mod tests {
    // wasm_bindgen_test_configure!(run_in_browser);

    use super::*;
    use crate::antfarm::elements::*;
    use bevy::utils::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Create a new application to be used for testing the gravity system.
    // Map and flatten a grid of elements and spawn associated elements into the world.
    fn setup(element_grid: Vec<Vec<Element>>) -> (App, HashMap<Position, Entity>) {
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        let spawned_elements: HashMap<_, _> = element_grid
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, element)| {
                        let position = Position::new(x as isize, y as isize);
                        let entity = app
                            .world
                            .spawn(ElementBundle::create(*element, position))
                            .id();

                        (position, entity)
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        let elements = spawned_elements.clone();

        let height = element_grid.len() as isize;
        let width = element_grid.first().map_or(0, |row| row.len()) as isize;
        let world_map = WorldMap::new(width, height, 0.0, Some(spawned_elements));
        app.world.insert_resource(world_map);

        (app, elements)
    }

    // Confirm that sand successfully falls downward through multiple tiles of air.
    #[wasm_bindgen_test]
    fn did_sand_fall_down() {
        // Arrange
        let element_grid = vec![vec![Element::Sand], vec![Element::Air], vec![Element::Air]];
        let (mut app, elements) = setup(element_grid);

        // Act
        app.update();
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::Y]),
            Some(&Position::ZERO)
        );
        assert_eq!(
            app.world.get::<Position>(elements[&Position::new(0, 2)]),
            Some(&Position::Y)
        );
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ZERO]),
            Some(&Position::new(0, 2))
        );
    }

    // Confirm that sand ontop of non-air stays put
    #[wasm_bindgen_test]
    fn did_sand_not_fall_down() {
        // Arrange
        let element_grid = vec![vec![Element::Sand], vec![Element::Dirt]];
        let (mut app, elements) = setup(element_grid);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ZERO]),
            Some(&Position::ZERO)
        );
        assert_eq!(
            app.world.get::<Position>(elements[&Position::Y]),
            Some(&Position::Y)
        );
    }

    // Confirm that sand falls properly to the left
    #[wasm_bindgen_test]
    fn did_sand_fall_left() {
        // Arrange
        let element_grid = vec![
            vec![Element::Air, Element::Sand],
            vec![Element::Air, Element::Dirt],
        ];
        let (mut app, elements) = setup(element_grid);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::X]),
            Some(&Position::Y)
        );
        assert_eq!(
            app.world.get::<Position>(elements[&Position::Y]),
            Some(&Position::X)
        );
    }

    // Confirm that sand falls properly to the right
    #[wasm_bindgen_test]
    fn did_sand_fall_right() {
        // Arrange
        let element_grid = vec![
            vec![Element::Sand, Element::Air],
            vec![Element::Dirt, Element::Air],
        ];
        let (mut app, elements) = setup(element_grid);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ZERO]),
            Some(&Position::ONE)
        );
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ONE]),
            Some(&Position::ZERO)
        );
    }

    // Confirm that sand does not fall to the left if blocked to its left

    // Confirm that sand does not fall to the right if blocked to its right

    // Confirm that sand does not fall to the left if blocked to its bottom-left

    // Confirm that sand does not fall to the right if blocked to its bottom-right

    // Confirm that coinflip occurs if sand can fall to both the left and right
}

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sand_gravity_system.in_schedule(CoreSchedule::FixedUpdate));
    }
}
