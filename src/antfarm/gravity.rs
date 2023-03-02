use bevy::prelude::*;

use super::{Active, Element};

// TODO: Add support for loosening neighboring sand.
// TODO: Add support for crushing deep sand.
// TODO: Add support for sand falling left/right randomly.
pub fn sand_gravity_system(mut elements_query: Query<(&Element, &mut Active, &mut Transform)>) {
    info!("Sand Gravity System Runs!");

    let (mut air_elements_query, mut sand_elements_query): (Vec<_>, Vec<_>) = elements_query
        .iter_mut()
        .filter(|(element, active, _)| {
            active.0 == true && (**element == Element::Air || **element == Element::Sand)
        })
        .partition(|&(element, _, _)| *element == Element::Air);

    info!("Air Count: {}!", air_elements_query.len());
    info!("Sand Count: {}", sand_elements_query.len());

    for (_, active, sand_transform) in sand_elements_query.iter_mut() {
        // Get the position beneath the sand and determine if it is air.
        let below_sand_translation = sand_transform.translation + Vec3::NEG_Y;

        // TODO: This seems wildly inefficient compared to my previous architecture. I'm searching all air elements just to check one, specific spot in the world.
        let air_below_sand = air_elements_query
            .iter_mut()
            .find(|(_, _, transform)| transform.translation == below_sand_translation)
            .map(|(_, _, transform)| transform);

        // If there is air below and sand above then swap the two
        if let Some(air_below_sand) = air_below_sand {
            air_below_sand.translation.y += 1.0;
            sand_transform.translation.y -= 1.0;
        } else {
            // Done falling, no longer active.
            active.0 = false;
        }
    }
}
