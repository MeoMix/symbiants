use bevy::prelude::*;

use super::{elements::Element, Position, WorldMap};
use rand::Rng;

// AffectedByGravity is just applied to Sand at the moment, but will try to make it work for Ant too.
// It's not applied to dirt to ensure tunnels don't collapse, but obviously this is nonsense.
// AffectedByGravity is, surprisingly, necessary to avoid overlapping queries in gravity system.
#[derive(Component)]
pub struct AffectedByGravity;

// Returns true if every element in `positions` is Element::Air
// NOTE: This returns true if given 0 positions.
fn is_all_air(
    world_map: &WorldMap,
    non_sand_query: &Query<(&Element, &mut Position), Without<AffectedByGravity>>,
    positions: Vec<Position>,
) -> bool {
    positions
        .iter()
        .map(|position| {
            let Some(&element) = world_map.elements.get(&position) else { return false; };
            let Ok((&element, _)) = non_sand_query.get(element) else { return false; };
            element == Element::Air
        })
        .all(|is_air| is_air)
}

// For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
// For each sand element which is above air, swap it with the air beneath it.
fn sand_gravity_system(
    mut sand_query: Query<&mut Position, (With<AffectedByGravity>, With<Element>)>,
    mut non_sand_query: Query<(&Element, &mut Position), Without<AffectedByGravity>>,
    world_map: Res<WorldMap>,
) {
    for mut sand_position in sand_query.iter_mut() {
        let mut go_left = false;
        let mut go_right = false;

        let below_sand_position = *sand_position + Position::Y;
        let left_sand_position = *sand_position + Position::NEG_X;
        let left_below_sand_position = *sand_position + Position::new(-1, 1);
        let right_sand_position = *sand_position + Position::X;
        let right_below_sand_position = *sand_position + Position::ONE;

        // If there is air below the sand then continue falling down.
        let go_down = is_all_air(&world_map, &non_sand_query, vec![below_sand_position]);

        // Otherwise, likely at rest, but potential for tipping off a precarious ledge.
        // Look for a column of air two units tall to either side of the sand and consider going in one of those directions.
        if !go_down {
            go_left = is_all_air(
                &world_map,
                &non_sand_query,
                vec![left_sand_position, left_below_sand_position],
            );

            go_right = is_all_air(
                &world_map,
                &non_sand_query,
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

        let mut target_position: Option<Position> = None;
        if go_down {
            target_position = Some(below_sand_position);
        } else if go_left {
            target_position = Some(left_below_sand_position);
        } else if go_right {
            target_position = Some(right_below_sand_position);
        }

        let Some(target_position) = target_position else { continue };
        let Some(&air_entity) = world_map.elements.get(&target_position) else { continue };
        let Ok((_, mut air_position)) = non_sand_query.get_mut(air_entity) else { continue };

        // Swap element positions internally.
        (sand_position.x, air_position.x) = (air_position.x, sand_position.x);
        (sand_position.y, air_position.y) = (air_position.y, sand_position.y);
    }
}

// TODO: logging during tests doesn't seem to work - wasm-pack vs trunk issue?
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::antfarm::elements::*;
    use bevy::utils::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Confirm that sand ontop of air falls downward.
    #[wasm_bindgen_test]
    fn did_sand_fall_down() {
        let mut app = App::new();

        let mut elements = HashMap::<Position, Entity>::new();

        let sand_position = Position::ZERO;
        let air_position = Position::Y;

        // Setup test entities
        let sand_id = app
            .world
            .spawn((
                ElementBundle::create_sand(Vec3::ZERO),
                sand_position,
                AffectedByGravity,
            ))
            .id();
        let air_id = app
            .world
            .spawn((ElementBundle::create_air(Vec3::NEG_Y), air_position))
            .id();

        elements.insert(sand_position, sand_id);
        elements.insert(air_position, air_id);

        app.world
            .insert_resource(WorldMap::new(1, 2, 0.0, Some(elements)));

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::Y);
        assert_eq!(app.world.get::<Position>(air_id).unwrap(), &Position::ZERO);
    }

    // Confirm that sand ontop of non-air stays put
    #[wasm_bindgen_test]
    fn did_sand_not_fall_down() {
        let mut app = App::new();

        let mut elements = HashMap::<Position, Entity>::new();

        let sand_position = Position::ZERO;
        let dirt_position = Position::Y;

        // Setup test entities
        let sand_id = app
            .world
            .spawn((
                ElementBundle::create_sand(Vec3::ZERO),
                sand_position,
                AffectedByGravity,
            ))
            .id();
        let dirt_id = app
            .world
            // TODO: The fact this NEG_Y has implicit relationship with air_position is no good
            .spawn((ElementBundle::create_dirt(Vec3::NEG_Y), dirt_position))
            .id();

        elements.insert(sand_position, sand_id);
        elements.insert(dirt_position, dirt_id);

        app.world
            .insert_resource(WorldMap::new(1, 2, 0.0, Some(elements)));

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::ZERO);
        assert_eq!(app.world.get::<Position>(dirt_id).unwrap(), &Position::Y);
    }

    // Confirm that sand at the bottom of the world doesn't panic
    #[wasm_bindgen_test]
    fn did_not_panic() {
        let mut app = App::new();

        let mut elements = HashMap::<Position, Entity>::new();

        let sand_position = Position::ZERO;

        // Setup test entities
        let sand_id = app
            .world
            .spawn((
                ElementBundle::create_sand(Vec3::ZERO),
                sand_position,
                AffectedByGravity,
            ))
            .id();

        elements.insert(sand_position, sand_id);

        app.world
            .insert_resource(WorldMap::new(1, 1, 0.0, Some(elements)));

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::ZERO);
    }

    // Confirm that sand falls properly to the left
    #[wasm_bindgen_test]
    fn did_sand_fall_left() {
        let mut app = App::new();

        let mut elements = HashMap::<Position, Entity>::new();

        let swapped_sand_position = Position::X;
        let swapped_air_position = Position::Y;

        let air_position = Position::ZERO;
        let dirt_position = Position::ONE;

        // Row 1
        let air_id = app
            .world
            .spawn((ElementBundle::create_air(Vec3::ZERO), air_position))
            .id();

        let swapped_sand_id = app
            .world
            .spawn((
                ElementBundle::create_sand(Vec3::X),
                swapped_sand_position,
                AffectedByGravity,
            ))
            .id();

        // Row 2
        let swapped_air_id = app
            .world
            .spawn((ElementBundle::create_air(Vec3::NEG_Y), swapped_air_position))
            .id();

        let dirt_id = app
            .world
            .spawn((ElementBundle::create_dirt(Vec3::NEG_ONE), dirt_position))
            .id();

        // Setup test entities
        elements.insert(air_position, air_id);
        elements.insert(swapped_sand_position, swapped_sand_id);
        elements.insert(swapped_air_position, swapped_air_id);
        elements.insert(dirt_position, dirt_id);

        app.world
            .insert_resource(WorldMap::new(2, 2, 0.0, Some(elements)));

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(
            app.world.get::<Position>(swapped_sand_id).unwrap(),
            &Position::Y
        );
        assert_eq!(
            app.world.get::<Position>(swapped_air_id).unwrap(),
            &Position::X
        );
    }

    // Confirm that sand falls properly to the right
    #[wasm_bindgen_test]
    fn did_sand_fall_right() {
        let mut app = App::new();

        let mut elements = HashMap::<Position, Entity>::new();

        let swapped_sand_position = Position::ZERO;
        let swapped_air_position = Position::ONE;

        let air_position = Position::X;
        let dirt_position = Position::Y;

        // Row 1
        let swapped_sand_id = app
            .world
            .spawn((
                ElementBundle::create_sand(Vec3::ZERO),
                swapped_sand_position,
                AffectedByGravity,
            ))
            .id();

        let air_id = app
            .world
            .spawn((ElementBundle::create_air(Vec3::X), air_position))
            .id();

        // Row 2
        let dirt_id = app
            .world
            .spawn((ElementBundle::create_dirt(Vec3::NEG_Y), dirt_position))
            .id();

        let swapped_air_id = app
            .world
            .spawn((
                ElementBundle::create_air(Vec3::new(1.0, -1.0, 0.0)),
                swapped_air_position,
            ))
            .id();

        // Setup test entities
        elements.insert(swapped_sand_position, swapped_sand_id);
        elements.insert(air_position, air_id);
        elements.insert(dirt_position, dirt_id);
        elements.insert(swapped_air_position, swapped_air_id);

        app.world
            .insert_resource(WorldMap::new(2, 2, 0.0, Some(elements)));

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(
            app.world.get::<Position>(swapped_sand_id).unwrap(),
            &Position::ONE
        );
        assert_eq!(
            app.world.get::<Position>(swapped_air_id).unwrap(),
            &Position::ZERO
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
