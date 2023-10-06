use crate::{
    common::{get_entity_from_id, Id},
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

pub fn ants_act(
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
    id_query: Query<(Entity, &Id)>,
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

        // TODO: drop ahead not where at?
        if inventory.0 != None && rng.f32() < settings.probabilities.random_drop {
            let target_element_entity = world_map.element_entity(*position);
            commands.drop(ant_entity, *position, *target_element_entity);
            continue;
        }

        let ahead_position = orientation.get_ahead_position(position);
        if !world_map.is_within_bounds(&ahead_position) {
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.get_element_entity(ahead_position).unwrap();
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element != Element::Air {
            // Consider digging / picking up the element under various circumstances.
            if inventory.0 == None {
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
                let dig_sand = *element == Element::Sand
                    && world_map.is_underground(&position)
                    && birthing.is_none();

                if dig_food || dig_sand {
                    let dig_position = orientation.get_ahead_position(position);
                    let dig_target_entity = *world_map.element_entity(dig_position);
                    commands.dig(ant_entity, dig_position, dig_target_entity);
                    continue;
                }
            }

            // Decided to not dig through and can't walk through
            continue;
        }

        // There is an air gap directly ahead of the ant. Consider dropping inventory.
        // Avoid dropping inventory when facing upwards since it'll fall on the ant.
        if inventory.0 != None && orientation.is_horizontal() {
            let inventory_item_element_id = inventory.0.clone().unwrap();
            let inventory_item_element_entity =
                get_entity_from_id(inventory_item_element_id, &id_query).unwrap();

            let inventory_item_element = elements_query.get(inventory_item_element_entity).unwrap();

            // Prioritize dropping sand above ground and food below ground.
            let drop_sand = *inventory_item_element == Element::Sand
                && world_map.is_aboveground(&ahead_position)
                && rng.f32() < settings.probabilities.above_surface_sand_drop;

            let mut drop_food = false;
            if *inventory_item_element == Element::Food {
                if world_map.is_underground(&ahead_position) {
                    drop_food = rng.f32() < settings.probabilities.below_surface_food_drop;

                    // If ant is adjacent to food then strongly consider dropping food (creates food piles)
                    let is_food_below = world_map.is_element(
                        &elements_query,
                        orientation.get_below_position(position),
                        Element::Food,
                    );
                    let is_food_ahead_below = world_map.is_element(
                        &elements_query,
                        orientation.get_below_position(&ahead_position),
                        Element::Food,
                    );

                    if (is_food_below || is_food_ahead_below)
                        && rng.f32() < settings.probabilities.below_surface_food_adjacent_food_drop
                    {
                        drop_food = true;
                    }
                } else {
                    if *role == AntRole::Queen {
                        drop_food =
                            rng.f32() < settings.probabilities.above_surface_queen_food_drop;
                    }
                }
            }

            if drop_sand || drop_food {
                // Drop inventory in front of ant
                let target_element_entity = world_map.element_entity(ahead_position);
                commands.drop(ant_entity, ahead_position, *target_element_entity);
                continue;
            }
        }
    }
}
