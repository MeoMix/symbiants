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
#[cfg(test)]
pub mod tests {
    // wasm_bindgen_test_configure!(run_in_browser);

    use super::*;
    use crate::antfarm::elements::*;
    use bevy::utils::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Confirm that sand successfully falls downward through multiple tiles of air.
    #[wasm_bindgen_test]
    fn did_sand_fall_down() {
        // Arrange
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        // Col 0 / Row 0
        let sand_position = Position::ZERO;
        let sand_id = app
            .world
            .spawn(ElementBundle::create_sand(sand_position))
            .id();

        // Col 0 / Row 1
        let air_position = Position::Y;
        let air_id = app
            .world
            .spawn(ElementBundle::create_air(air_position))
            .id();

        // Col 0 / Row 2
        let air_position2 = Position::new(0, 2);
        let air_id2 = app
            .world
            .spawn(ElementBundle::create_air(air_position2))
            .id();

        app.world.insert_resource(WorldMap::new(
            1,
            2,
            0.0,
            Some(HashMap::from([
                (sand_position, sand_id),
                (air_position, air_id),
                (air_position2, air_id2),
            ])),
        ));

        // Act
        app.update();
        app.update();

        // Assert
        assert_eq!(app.world.get::<Position>(air_id), Some(&Position::ZERO));
        assert_eq!(app.world.get::<Position>(air_id2), Some(&Position::Y));
        assert_eq!(
            app.world.get::<Position>(sand_id),
            Some(&Position::new(0, 2))
        );
    }

    // Confirm that sand ontop of non-air stays put
    #[wasm_bindgen_test]
    fn did_sand_not_fall_down() {
        // Arrange
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        // Col 0 / Row 0
        let sand_position = Position::ZERO;
        let sand_id = app
            .world
            .spawn(ElementBundle::create_sand(sand_position))
            .id();

        // Col 0 / Row 1
        let dirt_position = Position::Y;
        let dirt_id = app
            .world
            .spawn(ElementBundle::create_dirt(dirt_position))
            .id();

        app.world.insert_resource(WorldMap::new(
            1,
            2,
            0.0,
            Some(HashMap::from([
                (sand_position, sand_id),
                (dirt_position, dirt_id),
            ])),
        ));

        // Act
        app.update();

        // Assert
        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::ZERO);
        assert_eq!(app.world.get::<Position>(dirt_id).unwrap(), &Position::Y);
    }

    // Confirm that sand at the bottom of the world doesn't panic
    #[wasm_bindgen_test]
    fn did_not_panic() {
        // Arrange
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        // Col 0 / Row 0
        let sand_position = Position::ZERO;
        let sand_id = app
            .world
            .spawn(ElementBundle::create_sand(sand_position))
            .id();

        app.world.insert_resource(WorldMap::new(
            1,
            1,
            0.0,
            Some(HashMap::from([(sand_position, sand_id)])),
        ));

        // Act
        app.update();

        // Assert
        assert_eq!(app.world.get::<Position>(sand_id), Some(&Position::ZERO));
    }

    // Confirm that sand falls properly to the left
    #[wasm_bindgen_test]
    fn did_sand_fall_left() {
        // Arrange
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        // Col 0 / Row 0
        let air_position = Position::ZERO;
        let air_id = app
            .world
            .spawn(ElementBundle::create_air(air_position))
            .id();

        // Col 0 / Row 1
        let swapped_air_position = Position::Y;
        let swapped_air_id = app
            .world
            .spawn(ElementBundle::create_air(swapped_air_position))
            .id();

        // Col 1 / Row 0
        let swapped_sand_position = Position::X;
        let swapped_sand_id = app
            .world
            .spawn(ElementBundle::create_sand(swapped_sand_position))
            .id();

        // Col 1 / Row 1
        let dirt_position = Position::ONE;
        let dirt_id = app
            .world
            .spawn(ElementBundle::create_dirt(dirt_position))
            .id();

        app.world.insert_resource(WorldMap::new(
            2,
            2,
            0.0,
            Some(HashMap::from([
                (air_position, air_id),
                (swapped_sand_position, swapped_sand_id),
                (swapped_air_position, swapped_air_id),
                (dirt_position, dirt_id),
            ])),
        ));

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(swapped_sand_id),
            Some(&Position::Y)
        );
        assert_eq!(
            app.world.get::<Position>(swapped_air_id),
            Some(&Position::X)
        );
    }

    // Confirm that sand falls properly to the right
    #[wasm_bindgen_test]
    fn did_sand_fall_right() {
        // Arrange
        let mut app = App::new();
        app.add_system(sand_gravity_system);

        // Col 0 / Row 0
        let swapped_sand_position = Position::ZERO;
        let swapped_sand_id = app
            .world
            .spawn(ElementBundle::create_sand(swapped_sand_position))
            .id();

        // Col 0 / Row 1
        let dirt_position = Position::Y;
        let dirt_id = app
            .world
            .spawn(ElementBundle::create_dirt(dirt_position))
            .id();

        // Col 1 / Row 0
        let air_position = Position::X;
        let air_id = app
            .world
            .spawn(ElementBundle::create_air(air_position))
            .id();

        // Col 1 / Row 1
        let swapped_air_position = Position::ONE;
        let swapped_air_id = app
            .world
            .spawn((ElementBundle::create_air(swapped_air_position),))
            .id();

        // Setup test entities
        app.world.insert_resource(WorldMap::new(
            2,
            2,
            0.0,
            Some(HashMap::from([
                (swapped_sand_position, swapped_sand_id),
                (air_position, air_id),
                (dirt_position, dirt_id),
                (swapped_air_position, swapped_air_id),
            ])),
        ));

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(swapped_sand_id),
            Some(&Position::ONE)
        );
        assert_eq!(
            app.world.get::<Position>(swapped_air_id),
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
