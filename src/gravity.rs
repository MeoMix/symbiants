use crate::world_rng::WorldRng;

use super::{
    elements::{Element, ElementBundle},
    map::{Position, WorldMap},
    settings::Settings,
};
use bevy::prelude::*;
use itertools::{Either, Itertools};
use rand::{rngs::StdRng, Rng};

// TODO: Introduce more tests for sand crushing
// TODO: Add support for ant gravity
// PERF: could introduce 'active' component which isn't on everything, filter always, and not consider all elements all the time
// PERF: could make air more implicit and not represent it as an actual element to be iterated over.

// Returns true if every element in `positions` matches the provided Element type.
// NOTE: This returns true if given 0 positions.
fn is_all_element(
    world_map: &WorldMap,
    elements_query: &Query<(&Element, &mut Position)>,
    positions: Vec<Position>,
    search_element: Element,
) -> bool {
    positions
        .iter()
        .map(|position| {
            let Some(&element) = world_map.elements.get(&position) else { return false; };
            let Ok((&element, _)) = elements_query.get(element) else { return false; };
            element == search_element
        })
        .all(|is_air| is_air)
}

// Search for a valid location for sand to fall into by searching to the
// bottom left/center/right of a given sand position. Prioritize falling straight down
// and do not fall if surrounded by non-air
fn get_sand_fall_position(
    sand_position: Position,
    world_map: &WorldMap,
    elements_query: &Query<(&Element, &mut Position)>,
    world_rng: &mut StdRng,
) -> Option<Position> {
    // If there is air below the sand then continue falling down.
    let below_sand_position = sand_position + Position::Y;
    if is_all_element(
        &world_map,
        &elements_query,
        vec![below_sand_position],
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
        vec![left_sand_position, left_below_sand_position],
        Element::Air,
    );

    let right_sand_position = sand_position + Position::X;
    let right_below_sand_position = sand_position + Position::ONE;
    let mut go_right = is_all_element(
        &world_map,
        &elements_query,
        vec![right_sand_position, right_below_sand_position],
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
fn sand_gravity_system(
    mut elements_query: Query<(&Element, &mut Position)>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
) {
    let (sand_air_swaps, none_positions): (Vec<_>, Vec<_>) = elements_query
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
        ]) = elements_query.get_many_mut([air_entity, sand_entity]) else { continue };

        // Swap element positions internally.
        (sand_position.x, air_position.x) = (air_position.x, sand_position.x);
        (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

        // Update indices since they're indexed by position and track where elements are at.
        world_map.elements.insert(*sand_position, sand_entity);
        world_map.elements.insert(*air_position, air_entity);
    }

    let stationary_sand = elements_query
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
            above_sand_positions,
            Element::Sand,
        ) {
            world_map.elements.insert(
                *sand_position,
                commands
                    .spawn(ElementBundle::create(Element::Dirt, *sand_position))
                    .id(),
            );
        }
    }
}

// NOTE: To run just one test, run the command `cargo test <test_name>`
// TODO: Figure out headless testing (logging causes panic in node) and how to run single test
#[cfg(test)]
pub mod tests {
    use crate::world_rng::WorldRng;

    use super::*;
    use bevy::{log::LogPlugin, utils::HashMap};
    use rand::{rngs::StdRng, SeedableRng};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    // Create a new application to be used for testing the gravity system.
    // Map and flatten a grid of elements and spawn associated elements into the world.
    fn setup(
        element_grid: Vec<Vec<Element>>,
        seed: Option<u64>,
    ) -> (App, HashMap<Position, Entity>) {
        let mut app = App::new();
        // Not strictly necessary, but might as well keep info!("...")
        // in production code from causing panics when tested.
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

        let elements = spawned_elements.clone();

        let height = element_grid.len() as isize;
        let width = element_grid.first().map_or(0, |row| row.len()) as isize;
        let world_map = WorldMap::new(width, height, 0.0, Some(spawned_elements));
        app.insert_resource(world_map);
        app.insert_resource(Settings {
            world_width: width,
            world_height: height,
            ..default()
        });

        (app, elements)
    }

    // Confirm that sand successfully falls downward through multiple tiles of air.
    #[wasm_bindgen_test]
    fn did_sand_fall_down() {
        // Arrange
        let element_grid = vec![vec![Element::Sand], vec![Element::Air], vec![Element::Air]];
        let (mut app, elements) = setup(element_grid, None);

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
        let (mut app, elements) = setup(element_grid, None);

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
        let (mut app, elements) = setup(element_grid, None);

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
        let (mut app, elements) = setup(element_grid, None);

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

    // Confirm that sand falls to the left on a tie between l/r when given an appropriate random seed
    #[wasm_bindgen_test]
    fn did_sand_fall_left_by_chance() {
        // Arrange
        let element_grid = vec![
            vec![Element::Air, Element::Sand, Element::Air],
            vec![Element::Air, Element::Dirt, Element::Air],
        ];
        let (mut app, elements) = setup(element_grid, Some(3));

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::X]),
            Some(&Position::Y)
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
        let (mut app, elements) = setup(element_grid, Some(1));

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::X]),
            Some(&Position::new(2, 1))
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
        let (mut app, elements) = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::X]),
            Some(&Position::X)
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
        let (mut app, elements) = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::X]),
            Some(&Position::X)
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
        let (mut app, elements) = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ZERO]),
            Some(&Position::ZERO)
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
        let (mut app, elements) = setup(element_grid, None);

        // Act
        app.update();

        // Assert
        assert_eq!(
            app.world.get::<Position>(elements[&Position::ZERO]),
            Some(&Position::ZERO)
        );
    }

    // Confirm that coinflip occurs if sand can fall to both the left and right

    // Confirm that sand crushes at depth
}

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sand_gravity_system.in_schedule(CoreSchedule::FixedUpdate));
    }
}
