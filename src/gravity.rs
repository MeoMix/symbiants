use crate::{
    ant::AntOrientation,
    elements::{is_all_element, Crushable, DirtElementBundle},
    time::IsFastForwarding,
    world_rng::WorldRng,
};

use super::{
    elements::Element,
    map::{Position, WorldMap},
    settings::Settings,
};
use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};

// Sand becomes unstable temporarily when falling or adjacent to falling sand
// It becomes stable next frame. If all sand were always unstable then it'd act more like a liquid.
#[derive(Component)]
pub struct Unstable;

// TODO: How to do an exact match when running a test?
// TODO: Add tests for ant gravity
// TODO: It would be nice to be able to assert an entire map using shorthand like element_grid

// Search for a valid location for an element to fall into by searching to the
// bottom left/center/right of a given position. Prioritize falling straight down
// and do not fall if surrounded by non-air
fn get_element_fall_position(
    position: Position,
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    world_rng: &mut StdRng,
) -> Option<Position> {
    // If there is air below then continue falling down.
    let below_position = position + Position::Y;
    if is_all_element(
        &world_map,
        &elements_query,
        &[below_position],
        &Element::Air,
    ) {
        return Some(below_position);
    }

    // TODO: maybe don't always fall left/right even if possible to fall
    // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
    // Look for a column of air two units tall to either side and consider going in one of those directions.
    let left_position = position + Position::NEG_X;
    let left_below_position = position + Position::new(-1, 1);
    let mut go_left = is_all_element(
        &world_map,
        &elements_query,
        &[left_position, left_below_position],
        &Element::Air,
    );

    let right_position = position + Position::X;
    let right_below_position = position + Position::ONE;
    let mut go_right = is_all_element(
        &world_map,
        &elements_query,
        &[right_position, right_below_position],
        &Element::Air,
    );

    // Flip a coin and choose a direction randomly to resolve ambiguity in fall direction.
    if go_left && go_right {
        go_left = world_rng.gen_bool(0.5);
        go_right = !go_left;
    }

    if go_left {
        Some(left_below_position)
    } else if go_right {
        Some(right_below_position)
    } else {
        None
    }
}

pub fn element_gravity_system(
    mut element_position_queries: ParamSet<(
        Query<&Position, (With<Element>, With<Unstable>)>,
        Query<&mut Position, With<Element>>,
    )>,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    mut world_rng: ResMut<WorldRng>,
) {
    let element_air_swaps: Vec<_> = element_position_queries
        .p0()
        .iter()
        .filter_map(|&position| {
            get_element_fall_position(position, &world_map, &elements_query, &mut world_rng.0)
                .and_then(|air_position| {
                    Some((
                        *world_map.get_element(position)?,
                        *world_map.get_element(air_position)?,
                    ))
                })
        })
        .collect();

    // Swap element/air positions and update internal state to reflect the swap
    for &(element_entity, air_entity) in element_air_swaps.iter() {
        let mut element_position_query = element_position_queries.p1();

        let Ok([
            mut air_position,
            mut element_position
        ]) = element_position_query.get_many_mut([air_entity, element_entity]) else { continue };

        // Swap element positions internally.
        (element_position.x, air_position.x) = (air_position.x, element_position.x);
        (element_position.y, air_position.y) = (air_position.y, element_position.y);

        // Update indices since they're indexed by position and track where elements are at.
        world_map.set_element(*element_position, element_entity);
        world_map.set_element(*air_position, air_entity);
    }
}

// Ants can have air below them and not fall into it (unlike sand) because they can cling to the sides of sand and dirt.
// However, if they are clinging to sand/dirt, and that sand/dirt disappears, then they're out of luck and gravity takes over.
pub fn ant_gravity_system(
    mut ants_query: Query<(&AntOrientation, &mut Position)>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
) {
    for (orientation, mut position) in ants_query.iter_mut() {
        // Figure out foot direction
        let below_feet_position = *position + orientation.rotate_towards_feet().get_forward_delta();

        // TODO: There's a bug here - ant that rotates such that its feet are on the side of the world
        // and then has soil dug out from underneath it - hovers in the air. This could be fine,
        // but ants don't climb the walls right now, so there's a mismatch in behavior.
        let is_air_beneath_feet = is_all_element(
            &world_map,
            &elements_query,
            &[below_feet_position],
            &Element::Air,
        );

        if is_air_beneath_feet {
            let below_position = *position + Position::Y;
            let is_air_below = is_all_element(
                &world_map,
                &elements_query,
                &[below_position],
                &Element::Air,
            );

            if is_air_below {
                position.y = below_position.y;
            }
        }
    }
}

// TODO: wire up tests properly
pub fn gravity_crush_system(
    element_position_query: Query<(&Element, Ref<Position>, Entity), With<Crushable>>,
    elements_query: Query<&Element>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
    settings: Res<Settings>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    // TODO: prefer not skipping gravity_crush when fast-forwarding, but searching `compact_sand_depth` number of elements
    // is too slow. There's frequently ~600+ sand needing to be searched. A "pressure" system which calculates "pressure"
    // as it changes (and thus eliminates the need to search) would be faster, but much more complicated.
    if is_fast_forwarding.0 {
        return;
    }
    // TODO: this could benefit from par_iter, but would need to reenvision it a bit.
    for (element, position, entity) in element_position_query.iter() {
        // Find all stationary sand
        if *element != Element::Sand || position.is_changed() {
            continue;
        }

        // Crush sand that is under sufficient pressure
        let above_sand_positions: Vec<_> = (1..=settings.compact_sand_depth)
            .map(|y| Position::new(position.x, position.y - y))
            .collect();

        if is_all_element(
            &world_map,
            &elements_query,
            &above_sand_positions,
            &Element::Sand,
        ) {
            // Despawn the sand because it's been crushed into dirt and show the dirt by spawning a new element.
            commands.entity(entity).despawn();
            world_map.set_element(
                *position,
                commands.spawn(DirtElementBundle::new(*position)).id(),
            );
        }
    }
}

pub fn loosen_neighboring_sand(
    location: Position,
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    commands: &mut Commands,
) {
    // For a given position, get all positions adjacent with a radius of two.
    let mut adjacent_positions = Vec::new();
    for x in -2..=2 {
        for y in -2..=2 {
            if x == 0 && y == 0 {
                continue;
            }

            adjacent_positions.push(location + Position::new(x, y));
        }
    }

    // For each adjacent position, if the element at that position is sand, mark it as unstable.
    for position in adjacent_positions.iter() {
        if let Some(entity) = world_map.get_element(*position) {
            if let Ok(element) = elements_query.get(*entity) {
                if *element == Element::Sand {
                    commands.entity(*entity).insert(Unstable);
                }
            }
        }
    }
}

// TODO: I think there's a bug in this still where if there's two sand stacked ontop of one another
// and then there's a gap of air, one of the sands can become stuck in the air?

pub fn gravity_stability_system(
    sand_query: Query<(&Element, Ref<Position>, Entity), With<Unstable>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (element, position, entity) in sand_query.iter() {
        // TODO: technically sand is the only unstable element right now but keeping this as a safeguard
        if *element != Element::Sand {
            continue;
        }

        // Sand which fell in the current frame is still unstable and has potentially loosened its neighbors
        // So, mark all neighboring sand as unstable
        if position.is_changed() {
            loosen_neighboring_sand(*position, &world_map, &elements_query, &mut commands);
        } else {
            // Sand which has stopped falling is no longer unstable.
            commands.entity(entity).remove::<Unstable>();
        }
    }
}
// #[cfg(test)]
// pub mod ant_gravity_system_tests {
//     use crate::save::WorldSaveState;

//     use super::*;
//     use bevy::{log::LogPlugin, utils::HashMap};
//     use rand::{rngs::StdRng, SeedableRng};
//     use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

//     wasm_bindgen_test_configure!(run_in_browser);

//     fn setup(element_grid: Vec<Vec<Element>>) -> App {
//         let mut app = App::new();
//         app.add_plugin(LogPlugin::default());
//         app.add_system(ant_gravity_system);

//         let seed = 42069; // ayy lmao
//         let world_rng = WorldRng {
//             rng: StdRng::seed_from_u64(seed),
//         };

//         app.insert_resource(world_rng);

//         // TODO: probably reuse setup function between gravity and ant tests - maybe all tests
//         let spawned_elements: HashMap<_, _> = element_grid
//             .iter()
//             .enumerate()
//             .map(|(y, row)| {
//                 row.iter()
//                     .enumerate()
//                     .map(|(x, element)| {
//                         let position = Position::new(x as isize, y as isize);
//                         let entity = app
//                             .world
//                             .spawn(ElementBundle::create(*element, position))
//                             .id();

//                         (position, entity)
//                     })
//                     .collect::<Vec<_>>()
//             })
//             .flatten()
//             .collect();

//         let height = element_grid.len() as isize;
//         let width = element_grid.first().map_or(0, |row| row.len()) as isize;
//         let world_map = WorldMap::new(
//             width,
//             height,
//             0.0,
//             WorldSaveState::default(),
//             Some(spawned_elements),
//         );
//         app.insert_resource(world_map);
//         app.insert_resource(Settings {
//             world_width: width,
//             world_height: height,
//             ..default()
//         });

//         app
//     }

//     #[wasm_bindgen_test]
//     fn upright_ant_over_air_falls_down() {
//         // Arrange
//         // let element_grid = vec![vec![Element::Air], vec![Element::Air]];
//         // let mut app = setup(element_grid);
//     }

//     fn upright_ant_over_dirt_stays_put() {}

//     fn sideways_left_ant_standing_dirt_over_air_stays_put() {}

//     fn sideways_left_ant_standing_air_over_air_falls_down() {}

//     fn sideways_right_ant_standing_dirt_over_air_stays_put() {}

//     fn sideways_right_ant_standing_air_over_air_falls_down() {}

//     // TODO: This is sus. A sideways ant is able to cling to dirt, but if it starts falling, it should probably keep falling
//     // rather than exhibiting a super-ant ability to cling to dirt mid-fall.
//     fn sideways_falling_ant_grabs_dirt() {}
// }

// TODO: confirm elements are despawned not just that grid is correct
// #[cfg(test)]
// pub mod sand_gravity_system_tests {
//     use crate::{
//         elements::{AirElementBundle, SandElementBundle},
//         map::WorldSaveState,
//     };

//     use super::*;
//     use bevy::{log::LogPlugin, utils::HashMap};
//     use rand::{rngs::StdRng, SeedableRng};
//     use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

//     wasm_bindgen_test_configure!(run_in_browser);

//     // Create a new application to be used for testing the gravity system.
//     // Map and flatten a grid of elements and spawn associated elements into the world.
//     fn setup(element_grid: Vec<Vec<Element>>, seed: Option<u64>) -> App {
//         let mut app = App::new();
//         app.add_plugin(LogPlugin::default());
//         app.add_system(sand_gravity_system);

//         let seed = seed.unwrap_or(42069); // ayy lmao
//         let world_rng = WorldRng(StdRng::seed_from_u64(seed));

//         app.insert_resource(world_rng);

//         let spawned_elements: HashMap<_, _> = element_grid
//             .iter()
//             .enumerate()
//             .map(|(y, row)| {
//                 row.iter()
//                     .enumerate()
//                     .map(|(x, element)| {
//                         let position = Position::new(x as isize, y as isize);

//                         let entity = match element {
//                             Element::Air => app.world.spawn(AirElementBundle::new(position)).id(),
//                             Element::Dirt => app.world.spawn(DirtElementBundle::new(position)).id(),
//                             Element::Sand => app.world.spawn(SandElementBundle::new(position)).id(),
//                         };

//                         (position, entity)
//                     })
//                     .collect::<Vec<_>>()
//             })
//             .flatten()
//             .collect();

//         let height = element_grid.len() as isize;
//         let width = element_grid.first().map_or(0, |row| row.len()) as isize;
//         let world_map = WorldMap::new(
//             width,
//             height,
//             0,
//             WorldSaveState::default(),
//             Some(spawned_elements),
//         );
//         app.insert_resource(world_map);
//         app.insert_resource(Settings {
//             world_width: width,
//             world_height: height,
//             ..default()
//         });

//         app
//     }

//     // Confirm that sand successfully falls downward through multiple tiles of air.
//     #[wasm_bindgen_test]
//     fn did_sand_fall_down() {
//         // Arrange
//         let element_grid = vec![vec![Element::Sand], vec![Element::Air], vec![Element::Air]];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::ZERO]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::Y]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::new(0, 2)]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand ontop of non-air stays put
//     #[wasm_bindgen_test]
//     fn did_sand_not_fall_down() {
//         // Arrange
//         let element_grid = vec![vec![Element::Sand], vec![Element::Dirt]];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::ZERO]),
//             Some(&Element::Sand)
//         );
//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::Y]),
//             Some(&Element::Dirt)
//         );
//     }

//     // Confirm that sand falls properly to the left
//     #[wasm_bindgen_test]
//     fn did_sand_fall_left() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Air, Element::Sand],
//             vec![Element::Air, Element::Dirt],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::X]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::Y]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand falls properly to the right
//     #[wasm_bindgen_test]
//     fn did_sand_fall_right() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Sand, Element::Air],
//             vec![Element::Dirt, Element::Air],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::ZERO]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::ONE]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand falls to the left on a tie between l/r when given an appropriate random seed
//     #[wasm_bindgen_test]
//     fn did_sand_fall_left_by_chance() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Air, Element::Sand, Element::Air],
//             vec![Element::Air, Element::Dirt, Element::Air],
//         ];
//         let mut app = setup(element_grid, Some(3));

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::Y]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand falls to the right on a tie between l/r when given an appropriate random seed
//     #[wasm_bindgen_test]
//     fn did_sand_fall_right_by_chance() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Air, Element::Sand, Element::Air],
//             vec![Element::Air, Element::Dirt, Element::Air],
//         ];
//         let mut app = setup(element_grid, Some(1));

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::new(2, 1)]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand does not fall to the left if blocked to its upper-left
//     #[wasm_bindgen_test]
//     fn did_sand_not_fall_upper_left() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Dirt, Element::Sand],
//             vec![Element::Air, Element::Dirt],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::X]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand does not fall to the left if blocked to its lower-left
//     #[wasm_bindgen_test]
//     fn did_sand_not_fall_lower_left() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Air, Element::Sand],
//             vec![Element::Dirt, Element::Dirt],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(world_map.elements[&Position::X]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand does not fall to the right if blocked to its upper-right
//     #[wasm_bindgen_test]
//     fn did_sand_not_fall_upper_right() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Sand, Element::Dirt],
//             vec![Element::Dirt, Element::Air],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::ZERO]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that sand does not fall to the right if blocked to its lower-right
//     #[wasm_bindgen_test]
//     fn did_sand_not_fall_lower_right() {
//         // Arrange
//         let element_grid = vec![
//             vec![Element::Sand, Element::Air],
//             vec![Element::Dirt, Element::Dirt],
//         ];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::ZERO]),
//             Some(&Element::Sand)
//         );
//     }

//     // Confirm that a pillar of sand will compact the bottom into dirt
//     #[wasm_bindgen_test]
//     fn did_sand_column_compact() {
//         // Arrange
//         let element_grid = vec![vec![Element::Sand]; 16];
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::new(0, 15)]),
//             Some(&Element::Dirt)
//         );
//     }

//     // Confirm that a pillar of floating sand falls downward instead of compacting into dirt
//     #[wasm_bindgen_test]
//     fn did_floating_sand_column_not_compact() {
//         // Arrange
//         let mut element_grid = vec![vec![Element::Sand]; 16];
//         element_grid.push(vec![Element::Air]);
//         let mut app = setup(element_grid, None);

//         // Act
//         app.update();

//         // Assert
//         let Some(world_map) = app.world.get_resource::<WorldMap>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::new(0, 15)]),
//             Some(&Element::Air)
//         );

//         assert_eq!(
//             app.world
//                 .get::<Element>(world_map.elements[&Position::new(0, 16)]),
//             Some(&Element::Sand)
//         );
//     }
// }
