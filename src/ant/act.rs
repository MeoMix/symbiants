use crate::{
    ant::birthing::Birthing,
    common::{get_entity_from_id, Id},
    element::Element,
    grid::{position::Position, WorldMap},
    nest::Nest,
    settings::Settings,
};

use super::{commands::AntCommandsExt, AntInventory, AntOrientation, AntRole, Dead, Initiative};
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
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    id_query: Query<(Entity, &Id)>,
    world_map: Res<WorldMap>,
    mut nest: ResMut<Nest>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    for (orientation, inventory, mut initiative, position, role, ant_entity) in
        ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        // TODO: queen specific logic
        if *role == AntRole::Queen {
            if !world_map.is_below_surface(&position) && !nest.is_started() {
                if inventory.0 == None {
                    if rng.f32() < settings.probabilities.above_surface_queen_nest_dig {
                        // If x position is within 20% of world edge then don't dig there
                        let offset = settings.world_width / 5;
                        let is_too_near_left_edge = position.x < offset;
                        let is_too_near_right_edge = position.x > settings.world_width - offset;

                        if !is_too_near_left_edge && !is_too_near_right_edge {
                            let target_position =
                                *position + orientation.rotate_forward().get_forward_delta();
                            let target_element_entity = *world_map.element(target_position);
                            commands.dig(ant_entity, target_position, target_element_entity);

                            initiative.consume_action();

                            // TODO: technically this command could fail and I wouldn't want to mark nested?
                            // TODO: replace this with pheromones - queen should be able to find her way back to dig site via pheromones rather than
                            // enforcing nest generation probabilistically
                            nest.start(target_position);

                            continue;
                        }
                    }
                }
            }

            if position.y - world_map.surface_level() > 8 && !nest.is_completed() {
                // Check if the queen is sufficiently surounded by space while being deep underground and, if so, decide to start nesting.
                let left_position = *position + Position::NEG_X;
                let above_position = *position + Position::NEG_Y;
                let right_position = *position + Position::X;

                let has_valid_air_nest = world_map.is_all_element(
                    &elements_query,
                    &[left_position, *position, above_position, right_position],
                    Element::Air,
                );

                let below_position = *position + Position::Y;
                // Make sure there's stable place for ant child to be born
                let behind_position = *position + orientation.turn_around().get_forward_delta();
                let behind_below_position = behind_position + Position::Y;

                let has_valid_dirt_nest = world_map.is_all_element(
                    &elements_query,
                    &[below_position, behind_below_position],
                    Element::Dirt,
                );

                if has_valid_air_nest && has_valid_dirt_nest {
                    nest.complete();

                    // Spawn birthing component on QueenAnt
                    commands.entity(ant_entity).insert(Birthing::default());

                    if inventory.0 != None {
                        let target_position = *position + orientation.get_forward_delta();
                        let target_element_entity = world_map.element(target_position);
                        commands.drop(ant_entity, target_position, *target_element_entity);
                    }

                    initiative.consume_action();
                    continue;
                }
            }

            if nest.is_completed() {
                continue;
            }
        }

        // Add some randomness to worker behavior to make more lively, need to avoid applying this to queen because
        // too much randomness can kill her before she can nest
        if *role == AntRole::Worker {
            if inventory.0 == None {
                // Randomly dig downwards / perpendicular to current orientation
                if rng.f32() < settings.probabilities.random_dig
                // Only queen should dig initial nest tunnel / breach the surface.
                    && world_map.is_below_surface(&position)
                {
                    let target_position =
                        *position + orientation.rotate_forward().get_forward_delta();

                    if world_map.is_within_bounds(&target_position) {
                        let target_element_entity = *world_map.element(target_position);

                        let target_element = elements_query.get(target_element_entity);

                        // TODO: Don't copy/paste this logic around - make it more local to the idea of digging.
                        // The intent here is to prevent ants from digging through dirt such that they destroy their base by digging holes through walls.
                        let dig_dirt =
                            target_element.is_ok() && *target_element.unwrap() == Element::Dirt;
                        let mut allow_dig = true;

                        if dig_dirt {
                            // Don't allow digging through dirt underground if it would break through into another tunnel or break through the surface.
                            let forward_target_position =
                                target_position + orientation.rotate_forward().get_forward_delta();

                            if world_map.is_within_bounds(&forward_target_position) {
                                let forward_target_element_entity =
                                    world_map.element(forward_target_position);

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
                            commands.dig(ant_entity, target_position, target_element_entity);

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

        let forward_position = *position + orientation.get_forward_delta();

        if !world_map.is_within_bounds(&forward_position) {
            // Hit an edge - need to turn.
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.get_element(forward_position).unwrap();
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element != Element::Air {
            // Consider digging / picking up the element under various circumstances.
            if inventory.0 == None {
                // When above ground, prioritize picking up food
                let dig_food = *element == Element::Food && !world_map.is_below_surface(&position) && *role == AntRole::Worker;
                // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
                let dig_sand = *element == Element::Sand && world_map.is_below_surface(&position);
                // If digging would break through into an open area then don't do that.
                // IRL this would be done by sensing pressure on the dirt, but in game we allow ants to look one unit ahead.
                let mut dig_dirt = *element == Element::Dirt
                    && world_map.is_below_surface(&position)
                    && rng.f32() < settings.probabilities.below_surface_dirt_dig;

                if dig_dirt {
                    // Don't allow digging through dirt underground if it would break through into another tunnel or break through the surface.
                    let next_forward_position = forward_position + orientation.get_forward_delta();
                    let next_forward_element_entity = world_map.get_element(next_forward_position);

                    if next_forward_element_entity.is_some() {
                        let Ok(next_forward_element) =
                            elements_query.get(*next_forward_element_entity.unwrap())
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
                let mut queen_dig = *element == Element::Dirt
                    && world_map.is_below_surface(&position)
                    && *role == AntRole::Queen;

                if position.y - world_map.surface_level() > 8 {
                    if rng.f32() > settings.probabilities.below_surface_queen_nest_dig {
                        queen_dig = false;
                    }
                }

                if dig_food || dig_sand || dig_dirt || queen_dig {
                    let target_position = *position + orientation.get_forward_delta();
                    let target_element_entity = *world_map.element(target_position);
                    commands.dig(ant_entity, target_position, target_element_entity);

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
                && !world_map.is_below_surface(&forward_position)
                && rng.f32() < settings.probabilities.above_surface_sand_drop;

            let drop_food = *inventory_item_element == Element::Food
                && world_map.is_below_surface(&forward_position)
                && rng.f32() < settings.probabilities.below_surface_food_drop;

            if drop_sand || drop_food {
                // Drop inventory in front of ant
                let target_element_entity = world_map.element(forward_position);
                commands.drop(ant_entity, forward_position, *target_element_entity);

                initiative.consume_action();
                continue;
            }
        }
    }
}
