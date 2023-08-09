use crate::{
    ant::birthing::Birthing,
    element::Element,
    map::{Position, WorldMap},
    settings::Settings,
    world_rng::WorldRng,
};
 
use super::{commands::AntCommandsExt, Dead, AntInventory, AntOrientation, AntRole, Initiative};
use bevy::prelude::*;
use rand::Rng;

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
    mut world_map: ResMut<WorldMap>,
    settings: Res<Settings>,
    mut world_rng: ResMut<WorldRng>,
    mut commands: Commands,
) {
    for (orientation, inventory, mut initiative, position, role, ant_entity) in ants_query.iter_mut() {
        if !initiative.can_act() {
            continue;
        }
        
        // TODO: queen specific logic
        if *role == AntRole::Queen {
            if !world_map.is_below_surface(&position) && !world_map.has_started_nest() {
                if inventory.0 == None {
                    if world_rng.0.gen::<f32>()
                        < settings.probabilities.above_surface_queen_nest_dig
                    {
                        let target_position =
                            *position + orientation.rotate_forward().get_forward_delta();
                        let target_element_entity = *world_map.get_element_expect(target_position);
                        commands.dig(ant_entity, target_position, target_element_entity);

                        initiative.consume_action();

                        // TODO: replace this with pheromones - queen should be able to find her way back to dig site via pheromones rather than
                        // enforcing nest generation probabilistically
                        world_map.start_nest();

                        continue;
                    }
                }
            }

            if position.y - world_map.surface_level() > 8 && !world_map.is_nested() {
                // Check if the queen is sufficiently surounded by space while being deep underground and, if so, decide to start nesting.
                let left_position = *position + Position::NEG_X;
                let above_position = *position + Position::new(0, -1);
                let right_position = *position + Position::X;

                let has_valid_air_nest = world_map.is_all_element(
                    &elements_query,
                    &[left_position, *position, above_position, right_position],
                    Element::Air,
                );

                let below_position = *position + Position::new(0, 1);
                // Make sure there's stable place for ant child to be born
                let behind_position = *position + orientation.turn_around().get_forward_delta();
                let behind_below_position = behind_position + Position::new(0, 1);

                let has_valid_dirt_nest = world_map.is_all_element(
                    &elements_query,
                    &[below_position, behind_below_position],
                    Element::Dirt,
                );

                if has_valid_air_nest && has_valid_dirt_nest {
                    world_map.mark_nested();

                    // Spawn birthing component on QueenAnt
                    commands.entity(ant_entity).insert(Birthing::default());

                    if inventory.0 != None {
                        let target_position = *position + orientation.get_forward_delta();
                        let target_element_entity = world_map.get_element_expect(target_position);
                        commands.drop(ant_entity, target_position, *target_element_entity);
                    }

                    initiative.consume_action();
                    continue;
                }
            }

            if world_map.is_nested() {
                continue;
            }
        }

        // Add some randomness to worker behavior to make more lively, need to avoid applying this to queen because
        // too much randomness can kill her before she can nest
        if *role == AntRole::Worker {
            if inventory.0 == None {
                // Randomly dig downwards / perpendicular to current orientation
                if world_rng.0.gen::<f32>() < settings.probabilities.random_dig
                    && world_map.is_below_surface(&position)
                {
                    let target_position =
                        *position + orientation.rotate_forward().get_forward_delta();
                    let target_element_entity = *world_map.get_element_expect(target_position);
                    commands.dig(ant_entity, target_position, target_element_entity);

                    initiative.consume_action();
                    continue;
                }
            } else {
                if world_rng.0.gen::<f32>() < settings.probabilities.random_drop {
                    let target_element_entity = world_map.get_element_expect(*position);
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
                let dig_food = *element == Element::Food && !world_map.is_below_surface(&position);
                // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
                let dig_sand = *element == Element::Sand && world_map.is_below_surface(&position);
                let dig_dirt = *element == Element::Dirt
                    && world_map.is_below_surface(&position)
                    && world_rng.0.gen::<f32>() < settings.probabilities.below_surface_dirt_dig;

                // Once at sufficient depth it's not guaranteed that queen will want to continue digging deeper.
                // TODO: Make this a little less rigid - it's weird seeing always a straight line down 8.
                let mut queen_dig = *element == Element::Dirt
                    && world_map.is_below_surface(&position)
                    && *role == AntRole::Queen;

                if position.y - world_map.surface_level() > 8 {
                    if world_rng.0.gen::<f32>()
                        > settings.probabilities.below_surface_queen_nest_dig
                    {
                        queen_dig = false;
                    }
                }

                if dig_food || dig_sand || dig_dirt || queen_dig {
                    let target_position = *position + orientation.get_forward_delta();
                    let target_element_entity = *world_map.get_element_expect(target_position);
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
            // Prioritize dropping sand above ground and food below ground.
            let drop_sand = inventory.0 == Some(Element::Sand)
                && !world_map.is_below_surface(&forward_position)
                && world_rng.0.gen::<f32>() < settings.probabilities.above_surface_sand_drop;

            let drop_food = inventory.0 == Some(Element::Food)
                && world_map.is_below_surface(&forward_position)
                && world_rng.0.gen::<f32>() < settings.probabilities.below_surface_food_drop;

            if drop_sand || drop_food {
                // Drop inventory in front of ant
                let target_element_entity = world_map.get_element_expect(forward_position);
                commands.drop(ant_entity, forward_position, *target_element_entity);
                
                initiative.consume_action();
                continue;
            }
        }
    }
}
