use bevy::prelude::*;

use super::{
    elements::{AffectedByGravity, Element, Elements2D, Position},
    WorldState,
};

// TODO: Add support for loosening neighboring sand.
// TODO: Add support for crushing deep sand.
// TODO: Add support for sand falling left/right randomly.
pub fn sand_gravity_system(
    mut sand_query: Query<(&Element, &mut Position, &mut Transform), With<AffectedByGravity>>,
    mut non_sand_query: Query<
        (&Element, &mut Position, &mut Transform),
        Without<AffectedByGravity>,
    >,
    mut elements2d_query: Query<&mut Elements2D>,
    world_state: Res<WorldState>,
) {
    let mut elements2d = elements2d_query.single_mut();

    // Iterate over each sand element
    // For each sand element, look beneath it in the 2D array and determine if the element beneath it is air.
    // For each sand element which is above air, swap it with the air beneath it.
    let sand_query = sand_query
        .iter_mut()
        // TODO: this is safety since in the future ants will be affected by gravity
        .filter(|(element, _, _)| **element == Element::Sand);

    for (_, mut sand_position, mut sand_transform) in sand_query {
        let element_below_sand =
            elements2d.0[(sand_position.y + 1) * world_state.width + sand_position.x];

        // Use &Entity to look up element_below_sand reference
        if let Ok((element, mut air_position, mut air_transform)) =
            non_sand_query.get_mut(element_below_sand)
        {
            if *element == Element::Air {
                // Swap elements in 2D vector to ensure they stay consistent with position and translation
                elements2d.0.swap(
                    air_position.y * world_state.width + air_position.x,
                    sand_position.y * world_state.width + sand_position.x,
                );

                // TODO: It seems like a good idea to keep model/view concerns separate, but could drop position entirely and rely on translation.
                // Swap element positions
                (sand_position.y, air_position.y) = (air_position.y, sand_position.y);

                // TODO: I could swap the Vec references instead of updating y, but that seems like a bad idea.
                // Reflect the updated position visually
                sand_transform.translation.y = -(sand_position.y as f32);
                air_transform.translation.y = -(air_position.y as f32);
            }
        }
    }
}

// #[test]
// fn did_drop_sand() {
// let mut app = App::new();

// let world_state = WorldState {
//     width: 1,
//     height: 2,
//     surface_level: 1,
// };

// let mut elements_2d = Vec::with_capacity((world_state.width * world_state.height) as usize);

// app.insert_resource(world_state);

// // Add gravity system
// app.add_system(sand_gravity_system);

// // Setup test entities
// let sand_id = app
//     .world
//     .spawn((
//         ElementBundle::create_sand(Vec3::ZERO),
//         Position { x: 0, y: 0 },
//         AffectedByGravity,
//     ))
//     .id();
// let air_id = app
//     .world
//     .spawn((
//         ElementBundle::create_air(Vec3::new(0.0, -1.0, 0.0)),
//         Position { x: 0, y: 1 },
//     ))
//     .id();

// elements_2d.push(sand_id);
// elements_2d.push(air_id);

// let elements_2d_id = app.world.spawn(Elements2D(elements_2d)).id();

// // Run systems
// app.update();

// let updated_elements_2d = app.world.get::<Elements2D>(elements_2d_id);
// assert_eq!((updated_elements_2d.unwrap().0)[0], air_id);
// assert_eq!((updated_elements_2d.unwrap().0)[1], sand_id);

// TODO: Check translation and position on element
// }
