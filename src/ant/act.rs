use crate::{
    common::{get_entity_from_id, Id},
    element::Element,
    grid::{position::Position, WorldMap},
    settings::Settings,
};

use super::{
    birthing::Birthing, commands::AntCommandsExt, AntInventory, AntOrientation, AntRole, Dead,
    Initiative,
};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub fn ants_act(
    mut ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
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
    for (orientation, inventory, mut initiative, position, role, ant_entity, birthing) in
        ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        // Add some randomness to worker behavior to make more lively, need to avoid applying this to queen because
        // too much randomness can kill her before she can nest
        if *role == AntRole::Worker {
            if inventory.0 == None {
                // Randomly dig downwards / perpendicular to current orientation
                if rng.f32() < settings.probabilities.random_dig
                // Only queen should dig initial nest tunnel / breach the surface.
                    && world_map.is_underground(&position)
                {
                    let ahead_position = orientation.get_ahead_position(position);

                    if world_map.is_within_bounds(&ahead_position) {
                        let target_element_entity = *world_map.element(ahead_position);

                        let target_element = elements_query.get(target_element_entity);

                        // TODO: Don't copy/paste this logic around - make it more local to the idea of digging.
                        // The intent here is to prevent ants from digging through dirt such that they destroy their base by digging holes through walls.
                        let dig_dirt =
                            target_element.is_ok() && *target_element.unwrap() == Element::Dirt;
                        let mut allow_dig = true;

                        if dig_dirt {
                            // Don't allow digging through dirt underground if it would break through into another tunnel or break through the surface.
                            let ahead_ahead_position =
                                orientation.get_ahead_position(&ahead_position);

                            if world_map.is_within_bounds(&ahead_ahead_position) {
                                let forward_target_element_entity =
                                    world_map.element(ahead_ahead_position);

                                let Ok(forward_target_element) =
                                    elements_query.get(*forward_target_element_entity)
                                else {
                                    panic!("act - expected entity to exist")
                                };

                                // If the check here is for air then digging up through surface with sand stacked ontop is allowed
                                // which can result in ants undermining their own nest.
                                if *forward_target_element != Element::Dirt {
                                    allow_dig = false;
                                }
                            }
                        }

                        if allow_dig {
                            commands.dig(ant_entity, ahead_position, target_element_entity);

                            initiative.consume_action();
                            continue;
                        }
                    }
                }
            } else {
                if rng.f32() < settings.probabilities.random_drop {
                    let target_element_entity = world_map.element(*position);
                    commands.drop(ant_entity, *position, *target_element_entity);

                    initiative.consume_action();
                    continue;
                }
            }
        }

        let ahead_position = orientation.get_ahead_position(position);

        if !world_map.is_within_bounds(&ahead_position) {
            // Hit an edge - need to turn.
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.get_element(ahead_position).unwrap();
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element != Element::Air {
            // Consider digging / picking up the element under various circumstances.
            if inventory.0 == None {
                // When above ground, prioritize picking up food
                let dig_food = *element == Element::Food
                    && world_map.is_aboveground(&position)
                    && *role == AntRole::Worker;
                // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
                let dig_sand = *element == Element::Sand
                    && world_map.is_underground(&position)
                    && birthing.is_none();
                // If digging would break through into an open area then don't do that.
                // IRL this would be done by sensing pressure on the dirt, but in game we allow ants to look one unit ahead.
                let mut dig_dirt = *element == Element::Dirt
                    && world_map.is_underground(&position)
                    && rng.f32() < settings.probabilities.below_surface_dirt_dig
                    && birthing.is_none();

                if dig_dirt {
                    // Don't allow digging through dirt underground if it would break through into another tunnel or break through the surface.
                    let ahead_ahead_position = orientation.get_ahead_position(&ahead_position);
                    let ahead_ahead_element_entity = world_map.get_element(ahead_ahead_position);

                    if ahead_ahead_element_entity.is_some() {
                        let Ok(next_forward_element) =
                            elements_query.get(*ahead_ahead_element_entity.unwrap())
                        else {
                            panic!("act - expected entity to exist")
                        };

                        // If the check here is for air then digging up through surface with sand stacked ontop is allowed
                        // which can result in ants undermining their own nest.
                        if *next_forward_element != Element::Dirt {
                            dig_dirt = false;
                        }
                    }
                }

                // Once at sufficient depth it's not guaranteed that queen will want to continue digging deeper.
                // TODO: Make this a little less rigid - it's weird seeing always a straight line down 8.
                // TODO: support nest location getting unintentionally filled in by dirt/sand/food
                let mut queen_dig_nest = *element == Element::Dirt
                    && world_map.is_underground(&position)
                    && *role == AntRole::Queen;

                if position.y - world_map.surface_level() > 8 {
                    if rng.f32() > settings.probabilities.below_surface_queen_nest_dig {
                        queen_dig_nest = false;
                    }
                }

                if dig_food || dig_sand || dig_dirt || queen_dig_nest {
                    let dig_position = orientation.get_ahead_position(position);
                    let dig_target_entity = *world_map.element(dig_position);
                    commands.dig(ant_entity, dig_position, dig_target_entity);

                    initiative.consume_action();
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

            let drop_food = *inventory_item_element == Element::Food
                && world_map.is_underground(&ahead_position)
                && rng.f32() < settings.probabilities.below_surface_food_drop;

            if drop_sand || drop_food {
                // Drop inventory in front of ant
                let target_element_entity = world_map.element(ahead_position);
                commands.drop(ant_entity, ahead_position, *target_element_entity);

                initiative.consume_action();
                continue;
            }
        }
    }
}
