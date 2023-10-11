use crate::{
    element::Element,
    settings::Settings,
    world_map::{position::Position, WorldMap},
};

use super::{
    birthing::Birthing, commands::AntCommandsExt, AntInventory, AntOrientation, AntRole, Dead,
    Initiative,
};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub fn ants_dig(
    ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            &AntRole,
            Entity,
            Option<&Birthing>,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    for (orientation, inventory, initiative, position, role, ant_entity, birthing) in
        ants_query.iter()
    {
        if !initiative.can_act() {
            continue;
        }

        // Consider digging / picking up the element under various circumstances.
        if inventory.0 != None {
            continue;
        }

        let positions = [
            orientation.get_ahead_position(position),
            orientation.get_below_position(position),
            orientation.get_above_position(position)
        ];

        for position in positions {
            if try_dig(
                ant_entity,
                role,
                birthing,
                position,
                &elements_query,
                &world_map,
                &mut commands,
                &settings,
                &mut rng,
            ) {
                return;
            }
        }
    }
}

fn try_dig(
    ant_entity: Entity,
    ant_role: &AntRole,
    ant_birthing: Option<&Birthing>,
    dig_position: Position,
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
    commands: &mut Commands,
    settings: &Res<Settings>,
    rng: &mut ResMut<GlobalRng>,
) -> bool {
    if !world_map.is_within_bounds(&dig_position) {
        return false;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let element_entity = world_map.get_element_entity(dig_position).unwrap();
    let element = elements_query.get(*element_entity).unwrap();
    if *element == Element::Air {
        return false;
    }

    // When above ground, prioritize picking up food
    let mut dig_food = false;

    if *element == Element::Food && *ant_role == AntRole::Worker {
        if world_map.is_aboveground(&dig_position) {
            dig_food = rng.f32() < settings.probabilities.above_surface_food_dig;
        } else {
            dig_food = rng.f32() < settings.probabilities.below_surface_food_dig;
        }
    }

    // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
    let dig_sand = *element == Element::Sand
        && world_map.is_underground(&dig_position)
        && ant_birthing.is_none();

    if dig_food || dig_sand {
        commands.dig(ant_entity, dig_position, *element_entity);

        return true;
    }

    return false;
}
