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

        let ahead_position = orientation.get_ahead_position(position);
        if !world_map.is_within_bounds(&ahead_position) {
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.get_element_entity(ahead_position).unwrap();
        let element = elements_query.get(*entity).unwrap();
        if *element == Element::Air {
            continue;
        }

        // When above ground, prioritize picking up food
        let mut dig_food = false;

        if *element == Element::Food && *role == AntRole::Worker {
            if world_map.is_aboveground(&position) {
                dig_food = rng.f32() < settings.probabilities.above_surface_food_dig;
            } else {
                dig_food = rng.f32() < settings.probabilities.below_surface_food_dig;
            }
        }

        // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
        let dig_sand =
            *element == Element::Sand && world_map.is_underground(&position) && birthing.is_none();

        if dig_food || dig_sand {
            let dig_position = orientation.get_ahead_position(position);
            let dig_target_entity = *world_map.element_entity(dig_position);
            commands.dig(ant_entity, dig_position, dig_target_entity);
        }
    }
}
