use super::nest::AtNest;
use crate::{
    common::{
        ant::{initiative::Initiative, Dead, NestOrientation},
        element::{commands::ElementCommandsExt, Air, Element},
        grid::{Grid, GridElements, GridElementsMut},
        position::Position,
    },
    nest_simulation::nest::Nest,
    settings::Settings,
};
use bevy::{prelude::*, utils::HashSet};
use bevy_turborand::{DelegatedRng, GlobalRng};
use std::marker::PhantomData;

// Sand becomes unstable temporarily when falling or adjacent to falling sand
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Unstable;

// Don't infer Stable implicitly from lack of Unstable for performance.
// It's important for a System to be able to apply Unstable to newly added Elements without
// iterating all Element each run in an attempt to do so.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Stable;

pub fn register_gravity(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Unstable>();
    app_type_registry.write().register::<Stable>();
}

// TODO: How to do an exact match when running a test?
// TODO: Add tests for ant gravity
// TODO: It would be nice to be able to assert an entire map using shorthand like element_grid

// Search for a valid position for an element to fall into by searching to the
// bottom left/center/right of a given position. Prioritize falling straight down
// and do not fall if surrounded by non-air
fn get_element_fall_position(
    position: Position,
    grid_elements: &GridElements<AtNest>,
    rng: &mut Mut<GlobalRng>,
) -> Option<Position> {
    // If there is air below then continue falling down.
    let below_position = position + Position::Y;
    if grid_elements.is(below_position, Element::Air) {
        return Some(below_position);
    }

    // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
    // Look for a column of air two units tall to either side and consider going in one of those directions.
    let left_position = position + Position::NEG_X;
    let left_below_position = position + Position::new(-1, 1);
    let mut go_left = grid_elements.is_all(&[left_position, left_below_position], Element::Air)
        && rng.chance(0.66);

    let right_position = position + Position::X;
    let right_below_position = position + Position::ONE;
    let mut go_right = grid_elements.is_all(&[right_position, right_below_position], Element::Air)
        && rng.chance(0.66);

    // Flip a coin and choose a direction randomly to resolve ambiguity in fall direction.
    if go_left && go_right {
        go_left = rng.bool();
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

pub fn gravity_elements(
    mut element_position_queries: ParamSet<(
        Query<&Position, (With<Element>, With<Unstable>, With<AtNest>)>,
        Query<&mut Position, (With<Element>, With<AtNest>)>,
    )>,
    mut rng: ResMut<GlobalRng>,
    mut grid_elements_queries: ParamSet<(GridElements<AtNest>, GridElementsMut<AtNest>)>,
) {
    let element_air_swaps: Vec<_> = element_position_queries
        .p0()
        .iter()
        .filter_map(|&position| {
            get_element_fall_position(position, &grid_elements_queries.p0(), &mut rng.reborrow())
                .and_then(|air_position| {
                    Some((
                        *grid_elements_queries.p0().get_entity(position)?,
                        *grid_elements_queries.p0().get_entity(air_position)?,
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

        // Update indices since they're indexed by position and track where elements are at.
        grid_elements_queries
            .p1()
            .set(*element_position, element_entity);
        grid_elements_queries.p1().set(*air_position, air_entity);
    }
}

// Ants can have air below them and not fall into it (unlike sand) because they can cling to the sides of sand and dirt.
// However, if they are clinging to sand/dirt, and that sand/dirt disappears, then they're out of luck and gravity takes over.
pub fn gravity_ants(
    mut ants_query: Query<
        (
            &NestOrientation,
            &mut Position,
            Option<&mut Initiative>,
            Option<&Dead>,
        ),
        With<AtNest>,
    >,
    nest_query: Query<(&Grid, &Nest)>,
    grid_elements: GridElements<AtNest>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
) {
    let (grid, nest) = nest_query.single();

    for (orientation, mut position, initiative, dead) in ants_query.iter_mut() {
        // Figure out foot direction
        let below_position = orientation.get_below_position(&position);

        let is_air_beneath_feet = grid_elements.is_all(&[below_position], Element::Air);

        // SPECIAL CASE: out of bounds underground is considered dirt not air
        let is_out_of_bounds_beneath_feet =
            !grid.is_within_bounds(&below_position) && nest.is_aboveground(&below_position);

        let is_chance_falling =
            orientation.is_upside_down() && rng.f32() < settings.probabilities.random_fall;
        let is_chance_slipping =
            orientation.is_vertical() && rng.f32() < settings.probabilities.random_slip;
        // TODO: dead ants should be able to tumble to like sand/food
        let is_dead = dead.is_some();

        if is_air_beneath_feet
            || is_out_of_bounds_beneath_feet
            || is_chance_falling
            || is_chance_slipping
            || is_dead
        {
            let below_position = *position + Position::Y;
            let is_air_below = grid_elements.is_all(&[below_position], Element::Air);

            if is_air_below {
                position.y = below_position.y;

                // Ant falling through the air loses the ability to move or act.
                // Ants that are asleep don't have initiative
                if let Some(mut initiative) = initiative {
                    if initiative.can_act() {
                        initiative.consume();
                    }
                }
            }
        }
    }
}

// If an air gap appears on the grid (either through spawning or movement of air) then mark adjacent elements as unstable.
pub fn gravity_mark_unstable(
    air_query: Query<&Position, (With<Air>, Changed<Position>, With<AtNest>)>,
    mut commands: Commands,
    nest_query: Query<&Nest>,
    grid_elements: GridElements<AtNest>,
) {
    let mut positions = HashSet::new();

    for &position in air_query.iter() {
        positions.insert(position + Position::new(-1, -1));
        positions.insert(position + Position::new(0, -1));
        positions.insert(position + Position::new(1, -1));
    }

    let nest = nest_query.single();

    for &position in &positions {
        // If the current position contains a sand or food element, mark it as unstable
        if let Some(entity) = grid_elements.get_entity(position) {
            let element = grid_elements.element(*entity);

            // Special Case - dirt aboveground doesn't have "background" supporting dirt to keep it stable - so it falls.
            if matches!(*element, Element::Sand | Element::Food)
                || *element == Element::Dirt && nest.is_aboveground(&position)
            {
                commands.toggle_element_command(
                    *entity,
                    position,
                    true,
                    Unstable,
                    PhantomData::<AtNest>,
                );

                commands.toggle_element_command(
                    *entity,
                    position,
                    false,
                    Stable,
                    PhantomData::<AtNest>,
                );
            }
        }
    }
}

/// Elements which were Unstable, but didn't move this frame, are marked Stable by removing their Unstable marker.
/// FIXME: floating column of sand can result in sand being marked stable while in the air due to having sand directly beneath.
pub fn gravity_mark_stable(
    unstable_element_query: Query<
        (Ref<Position>, Entity),
        (With<Unstable>, With<Element>, With<AtNest>),
    >,
    mut commands: Commands,
) {
    for (position, entity) in unstable_element_query.iter() {
        if !position.is_changed() {
            commands.toggle_element_command(
                entity,
                *position,
                false,
                Unstable,
                PhantomData::<AtNest>,
            );
            commands.toggle_element_command(entity, *position, true, Stable, PhantomData::<AtNest>);
        }
    }
}

/// All Elements are expected to be marked as Stable or Unstable.
/// Don't rely on just Added<Element> without multiple Without<> filters because that'll require iterating all elements each frame.
pub fn gravity_set_stability(
    element_query: Query<
        (Entity, &Element, &Position),
        (Without<Unstable>, Without<Stable>, With<AtNest>),
    >,
    nest_query: Query<&Nest>,
    mut commands: Commands,
) {
    let nest = nest_query.single();

    // TODO: Consider for_each for perf
    for (entity, element, position) in element_query.iter() {
        match element {
            Element::Air => {
                commands.entity(entity).insert(Stable);
            }
            Element::Dirt => {
                // Dirt that spawns below surface level is not unstable but dirt that is above is unstable.
                if nest.is_underground(position) {
                    commands.entity(entity).insert(Stable);
                } else {
                    commands.entity(entity).insert(Unstable);
                }
            }
            // Any sand or food that has just appeared starts off unstable. They'll be marked Stable later.
            Element::Sand => {
                commands.entity(entity).insert(Unstable);
            }
            Element::Food => {
                commands.entity(entity).insert(Unstable);
            }
        }
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
//         let world_rng = Rng {
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
//         let nest = Nest::new(
//             width,
//             height,
//             0.0,
//             WorldSaveState::default(),
//             Some(spawned_elements),
//         );
//         app.insert_resource(nest);
//         app.insert_resource(Settings {
//             nest_width: width,
//             nest_height: height,
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
//         element::{ElementBundle, SandElementBundle},
//         grid::WorldSaveState,
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
//         let world_rng = Rng(StdRng::seed_from_u64(seed));

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
//                             Element::Air => app.world.spawn(ElementBundle::new(Element::Air, position)).id(),
//                             Element::Dirt => app.world.spawn(ElementBundle::new(Element::Dirt, position)).id(),
//                             Element::Sand => app.world.spawn(ElementBundle::new(Element::Sand, position)).id(),
//                         };

//                         (position, entity)
//                     })
//                     .collect::<Vec<_>>()
//             })
//             .flatten()
//             .collect();

//         let height = element_grid.len() as isize;
//         let width = element_grid.first().map_or(0, |row| row.len()) as isize;
//         let nest = Nest::new(
//             width,
//             height,
//             0,
//             WorldSaveState::default(),
//             Some(spawned_elements),
//         );
//         app.insert_resource(nest);
//         app.insert_resource(Settings {
//             nest_width: width,
//             nest_height: height,
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::ZERO]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::Y]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::new(0, 2)]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::ZERO]),
//             Some(&Element::Sand)
//         );
//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::Y]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::X]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::Y]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::ZERO]),
//             Some(&Element::Air)
//         );
//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::ONE]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::Y]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::new(2, 1)]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::X]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world.get::<Element>(nest.elements[&Position::X]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::ZERO]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::ZERO]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::new(0, 15)]),
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
//         let Some(nest) = app.world.get_resource::<Nest>() else { panic!() };

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::new(0, 15)]),
//             Some(&Element::Air)
//         );

//         assert_eq!(
//             app.world
//                 .get::<Element>(nest.elements[&Position::new(0, 16)]),
//             Some(&Element::Sand)
//         );
//     }
// }
