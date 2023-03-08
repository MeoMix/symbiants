use bevy::prelude::*;

use super::elements::{AffectedByGravity, Element, Position, WorldMap};

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

    // For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
    // For each sand element which is above air, swap it with the air beneath it.
    for (mut sand_position, mut sand_transform) in sand_query.iter_mut() {
        // TODO: am I supposed to deref like this? seems sus
        // If there is air below the sand then continue falling down.
        if let Some(&element_below_sand) = world_map.elements.get(&(*sand_position + Position::Y)) {
            if let Ok((&element, mut air_position, mut air_transform)) =
                non_sand_query.get_mut(element_below_sand) && element == Element::Air
            {
                // Swap element positions
                (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

                // Reflect the updated position visually
                sand_transform.translation.y = -(sand_position.y as f32);
                air_transform.translation.y = -(air_position.y as f32);
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

        assert_eq!(app.world.get::<Position>(sand_id).unwrap().y, 1);
        assert_eq!(app.world.get::<Position>(air_id).unwrap().y, 0);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation.y,
            -1.0
        );
        assert_eq!(
            app.world.get::<Transform>(air_id).unwrap().translation.y,
            0.0
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

        assert_eq!(app.world.get::<Position>(sand_id).unwrap().y, 0);
        assert_eq!(app.world.get::<Position>(dirt_id).unwrap().y, 1);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation.y,
            0.0
        );
        assert_eq!(
            app.world.get::<Transform>(dirt_id).unwrap().translation.y,
            -1.0
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

        assert_eq!(app.world.get::<Position>(sand_id).unwrap().y, 0);

        assert_eq!(
            app.world.get::<Transform>(sand_id).unwrap().translation.y,
            0.0
        );
    }
}
