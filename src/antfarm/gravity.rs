use bevy::prelude::*;

use super::{
    elements::{AffectedByGravity, Elements2D, Position},
    Element, WORLD_WIDTH,
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
            elements2d.0[(sand_position.y + 1) * WORLD_WIDTH + sand_position.x];

        // Use &Entity to look up element_below_sand reference
        if let Ok((element, mut air_position, mut air_transform)) =
            non_sand_query.get_mut(element_below_sand)
        {
            if *element == Element::Air {
                // Swap elements in 2D vector to ensure they stay consistent with position and translation
                elements2d.0.swap(
                    air_position.y * WORLD_WIDTH + air_position.x,
                    sand_position.y * WORLD_WIDTH + sand_position.x,
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
