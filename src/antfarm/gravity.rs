use bevy::prelude::*;

use super::elements::{Element, Position, WorldMap};
use rand::Rng;

// AffectedByGravity is just applied to Sand at the moment, but will try to make it work for Ant too.
// It's not applied to dirt to ensure tunnels don't collapse, but obviously this is nonsense.
// AffectedByGravity is, surprisingly, necessary to avoid overlapping queries in gravity system.
#[derive(Component)]
pub struct AffectedByGravity;

// Returns true if every element in `positions` is Element::Air
fn is_all_air(
    world_map: &WorldMap,
    non_sand_query: &Query<(&Element, &mut Position, &mut Transform), Without<AffectedByGravity>>,
    positions: Vec<Position>,
) -> bool {
    positions
        .iter()
        .map(|position| {
            let mut is_air = false;

            if let Some(&element) = world_map.elements.get(&position) {
                if let Ok((&element, _, _)) = non_sand_query.get(element) {
                    if element == Element::Air {
                        is_air = true;
                    }
                }
            }

            is_air
        })
        .all(|is_air| is_air)
}

// For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
// For each sand element which is above air, swap it with the air beneath it.
pub fn sand_gravity_system(
    mut sand_query: Query<
        (&mut Position, &mut Transform),
        (With<AffectedByGravity>, With<Element>),
    >,
    mut non_sand_query: Query<
        (&Element, &mut Position, &mut Transform),
        Without<AffectedByGravity>,
    >,
    world_map_query: Query<&WorldMap>,
) {
    let world_map = world_map_query.single();

    for (mut sand_position, mut sand_transform) in sand_query.iter_mut() {
        // TODO: enum + match
        let mut go_down = false;
        let mut go_left = false;
        let mut go_right = false;

        let below_sand_position = *sand_position + Position::Y;
        let left_sand_position = *sand_position + Position::NEG_X;
        let left_below_sand_position = *sand_position + Position::NEG_ONE;
        let right_sand_position = *sand_position + Position::NEG_X;
        let right_below_sand_position = *sand_position + Position::NEG_ONE;

        // If there is air below the sand then continue falling down.
        go_down = is_all_air(&world_map, &non_sand_query, vec![below_sand_position]);

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

        if let Some(target_position) = target_position {
            if let Some(&air_entity) = world_map.elements.get(&target_position) {
                if let Ok((_, mut air_position, mut air_transform)) =
                    non_sand_query.get_mut(air_entity)
                {
                    // Swap element positions internally.
                    (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

                    // Swap element positions visually.
                    (sand_transform.translation.y, air_transform.translation.y) =
                        (air_transform.translation.y, sand_transform.translation.y);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::antfarm::elements::*;
    use bevy::utils::HashMap;
    use wasm_bindgen_test::wasm_bindgen_test;

    // Confirm that sand ontop of air falls downward.
    #[wasm_bindgen_test]
    fn did_drop_sand() {
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

        app.world.spawn(WorldMap { elements });

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::Y);
        assert_eq!(app.world.get::<Position>(air_id).unwrap(), &Position::ZERO);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation,
            Vec3::NEG_Y
        );
        assert_eq!(
            app.world.get::<Transform>(air_id).unwrap().translation,
            Vec3::ZERO
        );
    }

    // Confirm that sand ontop of non-air stays put
    #[wasm_bindgen_test]
    fn did_not_drop_sand() {
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
            .spawn((ElementBundle::create_dirt(Vec3::NEG_Y), dirt_position))
            .id();

        elements.insert(sand_position, sand_id);
        elements.insert(dirt_position, dirt_id);

        app.world.spawn(WorldMap { elements });

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::ZERO);
        assert_eq!(app.world.get::<Position>(dirt_id).unwrap(), &Position::Y);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation,
            Vec3::ZERO
        );
        assert_eq!(
            app.world.get::<Transform>(dirt_id).unwrap().translation,
            Vec3::NEG_Y
        );
    }

    // Confirm that sand at the bottom of the world doesn't panic
    #[wasm_bindgen_test]
    fn did_respect_bounds() {
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

        app.world.spawn(WorldMap { elements });

        // Add gravity system
        app.add_system(sand_gravity_system);
        // Run systems
        app.update();

        assert_eq!(app.world.get::<Position>(sand_id).unwrap(), &Position::ZERO);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation,
            Vec3::ZERO
        );
    }
}
