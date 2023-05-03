use crate::{
    ant::{get_delta, get_rotated_angle, AntAngle, AntFacing},
    elements::is_all_element,
    world_rng::WorldRng,
};

use super::{
    elements::{Element, ElementBundle},
    map::{Position, WorldMap},
    settings::Settings,
};
use bevy::prelude::*;
use itertools::{Either, Itertools};
use rand::{rngs::StdRng, Rng};

// TODO: How to do an exact match when running a test?
// TODO: Add tests for ant gravity
// TODO: extra bonus points for finding an abstraction that unifies element and ant gravity
// TODO: It would be nice to be able to assert an entire map using shorthand like element_grid
// PERF: could introduce 'active' component which isn't on everything, filter always, and not consider all elements all the time

// Search for a valid location for sand to fall into by searching to the
// bottom left/center/right of a given sand position. Prioritize falling straight down
// and do not fall if surrounded by non-air
fn get_sand_fall_position(
    sand_position: Position,
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    world_rng: &mut StdRng,
) -> Option<Position> {
    // If there is air below the sand then continue falling down.
    let below_sand_position = sand_position + Position::Y;
    if is_all_element(
        &world_map,
        &elements_query,
        &vec![below_sand_position],
        Element::Air,
    ) {
        return Some(below_sand_position);
    }

    // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
    // Look for a column of air two units tall to either side of the sand and consider going in one of those directions.
    let left_sand_position = sand_position + Position::NEG_X;
    let left_below_sand_position = sand_position + Position::new(-1, 1);
    let mut go_left = is_all_element(
        &world_map,
        &elements_query,
        &vec![left_sand_position, left_below_sand_position],
        Element::Air,
    );

    let right_sand_position = sand_position + Position::X;
    let right_below_sand_position = sand_position + Position::ONE;
    let mut go_right = is_all_element(
        &world_map,
        &elements_query,
        &vec![right_sand_position, right_below_sand_position],
        Element::Air,
    );

    // Flip a coin and choose a direction randomly to resolve ambiguity in fall direction.
    if go_left && go_right {
        go_left = world_rng.gen_bool(0.5);
        go_right = !go_left;
    }

    if go_left {
        Some(left_below_sand_position)
    } else if go_right {
        Some(right_below_sand_position)
    } else {
        None
    }
}

// For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
// For each sand element which is above air, swap it with the air beneath it.
pub fn sand_gravity_system(
    mut element_position_query: Query<(&Element, &mut Position)>,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
) {
    let (sand_air_swaps, none_positions): (Vec<_>, Vec<_>) = element_position_query
        .iter()
        .filter(|(&element, _)| element == Element::Sand)
        .map(|(_, &sand_position)| {
            get_sand_fall_position(
                sand_position,
                &world_map,
                &elements_query,
                &mut world_rng.rng,
            )
            .and_then(|air_position| {
                Some((
                    *world_map.elements.get(&sand_position)?,
                    *world_map.elements.get(&air_position)?,
                ))
            })
            .map_or_else(|| Either::Right(sand_position), |swap| Either::Left(swap))
        })
        .partition_map(|x| x);

    for &(sand_entity, air_entity) in sand_air_swaps.iter() {
        let Ok([
            (_, mut air_position),
            (_, mut sand_position)
        ]) = element_position_query.get_many_mut([air_entity, sand_entity]) else { continue };

        // Swap element positions internally.
        (sand_position.x, air_position.x) = (air_position.x, sand_position.x);
        (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

        // Update indices since they're indexed by position and track where elements are at.
        world_map.elements.insert(*sand_position, sand_entity);
        world_map.elements.insert(*air_position, air_entity);
    }

    let stationary_sand = element_position_query
        .iter()
        .filter(|(_, position)| none_positions.contains(&position));

    for (_, sand_position) in stationary_sand {
        // At deep enough levels, stationary sand finds itself crushed back into dirt.
        let start = 1;
        let end = settings.compact_sand_depth;
        let above_sand_positions: Vec<_> = (start..=end)
            .map(|y| Position::new(sand_position.x, sand_position.y - y))
            .collect();

        if is_all_element(
            &world_map,
            &elements_query,
            &above_sand_positions,
            Element::Sand,
        ) {
            // Despawn the sand because it's been crushed into dirt and show the dirt by spawning a new element.
            let crushed_sand_entity = world_map.elements.get(&sand_position).unwrap();
            commands.entity(*crushed_sand_entity).despawn();

            let entity = commands
                .spawn(ElementBundle::create(Element::Dirt, *sand_position))
                .id();

            world_map.elements.insert(*sand_position, entity);
        }
    }
}

// Ants can have air below them and not fall into it (unlike sand) because they can cling to the sides of sand and dirt.
// However, if they are clinging to sand/dirt, and that sand/dirt disappears, then they're out of luck and gravity takes over.
pub fn ant_gravity_system(
    mut ants_query: Query<(&AntFacing, &AntAngle, &mut Position)>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
) {
    for (facing, angle, mut position) in ants_query.iter_mut() {
        // Figure out foot direction
        let rotation = if *facing == AntFacing::Left { -1 } else { 1 };
        let foot_delta = get_delta(*facing, get_rotated_angle(*angle, rotation));
        let below_feet_position = *position + foot_delta;

        let is_air_beneath_feet = is_all_element(
            &world_map,
            &elements_query,
            &vec![below_feet_position],
            Element::Air,
        );

        if is_air_beneath_feet {
            let below_position = *position + Position::Y;
            let is_air_below = is_all_element(
                &world_map,
                &elements_query,
                &vec![below_position],
                Element::Air,
            );

            if is_air_below {
                position.y = below_position.y;
            }
        }
    }
}

#[cfg(test)]
pub mod ant_gravity_system_tests {
    use super::*;
    use bevy::{log::LogPlugin, utils::HashMap};
    use rand::{rngs::StdRng, SeedableRng};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    fn setup(element_grid: Vec<Vec<Element>>) -> App {
        let mut app = App::new();
        app.add_plugin(LogPlugin::default());
        app.add_system(ant_gravity_system);

        let seed = 42069; // ayy lmao
        let world_rng = WorldRng {
            rng: StdRng::seed_from_u64(seed),
        };

        app.insert_resource(world_rng);

        // TODO: probably reuse setup function between gravity and ant tests - maybe all tests
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

        let height = element_grid.len() as isize;
        let width = element_grid.first().map_or(0, |row| row.len()) as isize;
        let world_map = WorldMap::new(width, height, 0.0, Some(spawned_elements));
        app.insert_resource(world_map);
        app.insert_resource(Settings {
            world_width: width,
            world_height: height,
            ..default()
        });

        app
    }

    #[wasm_bindgen_test]
    fn upright_ant_over_air_falls_down() {
        // Arrange
        let element_grid = vec![vec![Element::Air], vec![Element::Air]];
        let mut app = setup(element_grid);
    }

    fn upright_ant_over_dirt_stays_put() {}

    fn sideways_left_ant_standing_dirt_over_air_stays_put() {}

    fn sideways_left_ant_standing_air_over_air_falls_down() {}

    fn sideways_right_ant_standing_dirt_over_air_stays_put() {}

    fn sideways_right_ant_standing_air_over_air_falls_down() {}

    // TODO: This is sus. A sideways ant is able to cling to dirt, but if it starts falling, it should probably keep falling
    // rather than exhibiting a super-ant ability to cling to dirt mid-fall.
    fn sideways_falling_ant_grabs_dirt() {}
}

// TODO: confirm elements are despawned not just that grid is correct
#[cfg(test)]
pub mod sand_gravity_system_tests {
    use super::*;
    use bevy::{log::LogPlugin, utils::HashMap};
    use rand::{rngs::StdRng, SeedableRng};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    // Create a new application to be used for testing the gravity system.
    // Map and flatten a grid of elements and spawn associated elements into the world.
    fn setup(element_grid: Vec<Vec<Element>>, seed: Option<u64>) -> App {
        let mut app = App::new();
        app.add_plugin(LogPlugin::default());
        app.add_system(sand_gravity_system);

        let seed = seed.unwrap_or(42069); // ayy lmao
        let world_rng = WorldRng {
            rng: StdRng::seed_from_u64(seed),
        };

        app.insert_resource(world_rng);

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

        let height = element_grid.len() as isize;
        let width = element_grid.first().map_or(0, |row| row.len()) as isize;
        let world_map = WorldMap::new(width, height, 0.0, Some(spawned_elements));
        app.insert_resource(world_map);
        app.insert_resource(Settings {
            world_width: width,
            world_height: height,
            ..default()
        });

        app
    }

    // Confirm that sand successfully falls downward through multiple tiles of air.
    #[wasm_bindgen_test]
    fn did_sand_fall_down() {
        // Arrange
        let element_grid = vec![vec![Element::Sand], vec![Element::Air], vec![Element::Air]];
        let mut app = setup(element_grid, None);

        // Act
        app.update();
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::ZERO]),
            Some(&Element::Air)
        );
        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::Y]),
            Some(&Element::Air)
        );
        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::new(0, 2)]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand ontop of non-air stays put
    #[wasm_bindgen_test]
    fn did_sand_not_fall_down() {
        // Arrange
        let element_grid = vec![vec![Element::Sand], vec![Element::Dirt]];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::ZERO]),
            Some(&Element::Sand)
        );
        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::Y]),
            Some(&Element::Dirt)
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
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::X]),
            Some(&Element::Air)
        );
        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::Y]),
            Some(&Element::Sand)
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
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::ZERO]),
            Some(&Element::Air)
        );
        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::ONE]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand falls to the left on a tie between l/r when given an appropriate random seed
    #[wasm_bindgen_test]
    fn did_sand_fall_left_by_chance() {
        // Arrange
        let element_grid = vec![
            vec![Element::Air, Element::Sand, Element::Air],
            vec![Element::Air, Element::Dirt, Element::Air],
        ];
        let mut app = setup(element_grid, Some(3));

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::Y]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand falls to the right on a tie between l/r when given an appropriate random seed
    #[wasm_bindgen_test]
    fn did_sand_fall_right_by_chance() {
        // Arrange
        let element_grid = vec![
            vec![Element::Air, Element::Sand, Element::Air],
            vec![Element::Air, Element::Dirt, Element::Air],
        ];
        let mut app = setup(element_grid, Some(1));

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::new(2, 1)]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand does not fall to the left if blocked to its upper-left
    #[wasm_bindgen_test]
    fn did_sand_not_fall_upper_left() {
        // Arrange
        let element_grid = vec![
            vec![Element::Dirt, Element::Sand],
            vec![Element::Air, Element::Dirt],
        ];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::X]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand does not fall to the left if blocked to its lower-left
    #[wasm_bindgen_test]
    fn did_sand_not_fall_lower_left() {
        // Arrange
        let element_grid = vec![
            vec![Element::Air, Element::Sand],
            vec![Element::Dirt, Element::Dirt],
        ];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world.get::<Element>(world_map.elements[&Position::X]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand does not fall to the right if blocked to its upper-right
    #[wasm_bindgen_test]
    fn did_sand_not_fall_upper_right() {
        // Arrange
        let element_grid = vec![
            vec![Element::Sand, Element::Dirt],
            vec![Element::Dirt, Element::Air],
        ];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::ZERO]),
            Some(&Element::Sand)
        );
    }

    // Confirm that sand does not fall to the right if blocked to its lower-right
    #[wasm_bindgen_test]
    fn did_sand_not_fall_lower_right() {
        // Arrange
        let element_grid = vec![
            vec![Element::Sand, Element::Air],
            vec![Element::Dirt, Element::Dirt],
        ];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::ZERO]),
            Some(&Element::Sand)
        );
    }

    // Confirm that a pillar of sand will compact the bottom into dirt
    #[wasm_bindgen_test]
    fn did_sand_column_compact() {
        // Arrange
        let element_grid = vec![vec![Element::Sand]; 16];
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::new(0, 15)]),
            Some(&Element::Dirt)
        );
    }

    // Confirm that a pillar of floating sand falls downward instead of compacting into dirt
    #[wasm_bindgen_test]
    fn did_floating_sand_column_not_compact() {
        // Arrange
        let mut element_grid = vec![vec![Element::Sand]; 16];
        element_grid.push(vec![Element::Air]);
        let mut app = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::new(0, 15)]),
            Some(&Element::Air)
        );

        assert_eq!(
            app.world
                .get::<Element>(world_map.elements[&Position::new(0, 16)]),
            Some(&Element::Sand)
        );
    }
}
