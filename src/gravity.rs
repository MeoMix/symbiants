use crate::{
    ant::AntOrientation,
    element::{commands::ElementCommandsExt, Air, Crushable},
    time::IsFastForwarding,
    world_rng::WorldRng,
};

use super::{
    element::Element,
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
    if world_map.is_element(&elements_query, below_position, Element::Air) {
        return Some(below_position);
    }

    // TODO: maybe don't always fall left/right even if possible to fall
    // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
    // Look for a column of air two units tall to either side and consider going in one of those directions.
    let left_position = position + Position::NEG_X;
    let left_below_position = position + Position::new(-1, 1);
    let mut go_left = world_map.is_all_element(
        &elements_query,
        &[left_position, left_below_position],
        Element::Air,
    ) && world_rng.gen_bool(0.33);

    let right_position = position + Position::X;
    let right_below_position = position + Position::ONE;
    let mut go_right = world_map.is_all_element(
        &elements_query,
        &[right_position, right_below_position],
        Element::Air,
    ) && world_rng.gen_bool(0.33);

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

pub fn element_gravity(
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

        let Ok([mut air_position, mut element_position]) =
            element_position_query.get_many_mut([air_entity, element_entity])
        else {
            continue;
        };

        // Swap element positions internally.
        (element_position.x, air_position.x) = (air_position.x, element_position.x);
        (element_position.y, air_position.y) = (air_position.y, element_position.y);

        // TODO: It seems wrong that when swapping two existing elements I need to manually update the world map
        // but that when spawning new elements the on_spawn_element system does it for me.
        // Update indices since they're indexed by position and track where elements are at.
        world_map.set_element(*element_position, element_entity);
        world_map.set_element(*air_position, air_entity);
    }
}

// Ants can have air below them and not fall into it (unlike sand) because they can cling to the sides of sand and dirt.
// However, if they are clinging to sand/dirt, and that sand/dirt disappears, then they're out of luck and gravity takes over.
pub fn ant_gravity(
    mut ants_query: Query<(&AntOrientation, &mut Position)>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
) {
    for (orientation, mut position) in ants_query.iter_mut() {
        // Figure out foot direction
        let below_feet_position = *position + orientation.rotate_forward().get_forward_delta();

        let is_air_beneath_feet =
            world_map.is_all_element(&elements_query, &[below_feet_position], Element::Air);

        let is_out_of_bounds = !world_map.is_within_bounds(&below_feet_position);

        if is_air_beneath_feet || is_out_of_bounds {
            let below_position = *position + Position::Y;
            let is_air_below =
                world_map.is_all_element(&elements_query, &[below_position], Element::Air);

            if is_air_below {
                position.y = below_position.y;
            }
        }
    }
}

// TODO: wire up tests properly
pub fn gravity_crush(
    element_position_query: Query<(&Element, Ref<Position>, Entity), With<Crushable>>,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
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

        if world_map.is_all_element(&elements_query, &above_sand_positions, Element::Sand) {
            // Despawn the sand because it's been crushed into dirt and show the dirt by spawning a new element.
            info!("replace_element5: {:?}", position);
            commands.replace_element(*position, entity, Element::Dirt);
        }
    }
}

// FIXME: There are bugs in the sand fall logic because gravity isn't processed from the bottom row up.
// A column of sand, floating in the air, may have some sand be marked stable while floating in the air due to having sand directly beneath.
pub fn gravity_stability(
    air_query: Query<Ref<Position>, (With<Air>, With<Element>)>,
    unstable_element_query: Query<(Ref<Position>, Entity), (With<Unstable>, With<Element>)>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    // If an air gap appears on the grid (either through spawning or movement of air) then mark adjacent elements as unstable.
    for position in air_query.iter().filter(|p| p.is_added() || p.is_changed()) {
        // Calculate the positions of the elements above the current air element
        let adjacent_positions = (-1..=1)
            .map(|x_offset| *position + Position::new(x_offset, -1))
            .collect::<Vec<_>>();

        // Iterate over all the calculated positions
        for &adjacent_position in &adjacent_positions {
            // If the current position contains a sand or food element, mark it as unstable
            if let Some(entity) = world_map.get_element(adjacent_position) {
                if let Ok(element) = elements_query.get(*entity) {
                    if matches!(*element, Element::Sand | Element::Food) {
                        commands.toggle_element_unstable(*entity, adjacent_position, true);
                    }
                }
            }
        }
    }

    // Iterate over all unstable elements that have not changed position
    for (position, entity) in unstable_element_query
        .iter()
        .filter(|(p, _)| !p.is_changed())
    {
        commands.toggle_element_unstable(entity, *position, false);
    }
}

// #[cfg(test)]
// pub mod ant_gravity_tests {
//     use crate::save::WorldSaveState;

//     use super::*;
//     use bevy::{log::LogPlugin, utils::HashMap};
//     use rand::{rngs::StdRng, SeedableRng};
//     use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

//     wasm_bindgen_test_configure!(run_in_browser);

//     fn setup(element_grid: Vec<Vec<Element>>) -> App {
//         let mut app = App::new();
//         app.add_plugin(LogPlugin::default());
//         app.add_systems(ant_gravity);

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
// pub mod sand_gravity_tests {
//     use crate::{
//         element::{AirElementBundle, SandElementBundle},
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
//         app.add_systems(sand_gravity);

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
