use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use crate::{
    ant::{
        commands::AntCommandsExt, walk::get_turned_orientation, AntInventory, AntOrientation, Dead,
        Initiative,
    },
    element::Element,
    pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneMap},
    world_map::{position::Position, WorldMap},
};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Tunneling(pub isize);

// "Whenever ant walks over tile with nesting pheromone, they gain "Nesting: 8". Then, they attempt to take a step forward and decrement Nesting to 7. If they end up digging, nesting is "forgotten" and they shift back to hauling dirt
// so they haul it out, then walk back, hit the pheromone again, and repeatedly try to take 8 steps
// and if they succeed in getting to nesting 0, then drop a new pheromone marker "build chamber here"
// and when they hit that marker, they will look around them in all directions and, if they see any adjacent dirt, they will dig one piece of it and go back into "haul dirt" mode
// if they do not see any adjacent dirt, they clean up the pheromone and, in the case of the queen, shift to giving birth
pub fn ants_tunnel_pheromone_move(
    mut ants_query: Query<
        (&mut AntOrientation, &mut Initiative, &mut Position),
        (Without<Dead>, With<Tunneling>),
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut orientation, mut initiative, mut ant_position) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // Ants that are following a tunneling pheromone will prioritize walking into open space in front of them
        // over digging directly in front of them.

        // If there's solid material in front of ant then consider turning onto it if there's tunnel to follow upward.
        let ahead_position = orientation.get_ahead_position(&ant_position);
        let has_air_ahead = world_map
            .get_element(ahead_position)
            .map_or(false, |entity| {
                elements_query
                    .get(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        let above_position = orientation.get_above_position(&ant_position);
        let has_air_above = world_map
            .get_element(above_position)
            .map_or(false, |entity| {
                elements_query
                    .get(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        if !has_air_ahead && has_air_above {
            *orientation = get_turned_orientation(
                &orientation,
                &ant_position,
                &elements_query,
                &world_map,
                &mut rng,
            );

            initiative.consume_movement();
            continue;
        }

        // Blocked, defer to default action
        if !has_air_ahead && !has_air_above {
            continue;
        }

        // Definitely walking forward, but if that results in standing over air then turn on current block.
        let foot_orientation = orientation.rotate_forward();
        let foot_position = foot_orientation.get_ahead_position(&ahead_position);

        if let Some(foot_entity) = world_map.get_element(foot_position) {
            let foot_element = elements_query.get(*foot_entity).unwrap();

            if *foot_element == Element::Air {
                // If ant moves straight forward, it will be standing over air. Instead, turn into the air and remain standing on current block
                *ant_position = foot_position;
                *orientation = foot_orientation;
            } else {
                // Just move forward
                *ant_position = ahead_position;
            }

            initiative.consume_movement();
        }
    }
}

pub fn ants_tunnel_pheromone_act(
    mut ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
            &Position,
            Entity,
        ),
        (Without<Dead>, With<Tunneling>),
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
) {
    for (orientation, inventory, mut initiative, position, ant_entity) in ants_query.iter_mut() {
        if !initiative.can_act() {
            continue;
        }

        // Safeguard, but not expected to run because shouldn't have Tunneling pheromone with full inventory.
        if inventory.0 != None {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(position);

        if !world_map.is_within_bounds(&ahead_position) {
            // Hit an edge - need to turn.
            error!("tunneled into wall - need to figure out how to handle this better still");
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = world_map.element(ahead_position);
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element == Element::Air {
            continue;
        }

        // TODO: Make this a little less rigid - it's weird seeing always a straight line down 8.
        // TODO: prefer only going straight or down when digging tunnels.
        // rng.f32() < settings.probabilities.below_surface_queen_nest_dig;
        let dig_position = orientation.get_ahead_position(position);
        let dig_target_entity = *world_map.element(dig_position);
        commands.dig(ant_entity, dig_position, dig_target_entity);
        initiative.consume_action();
    }
}

/// Apply tunneling to ants which walk over tiles covered in tunnel pheromone.
/// Tunneling is set to Tunneling(8). This encourages ants to prioritize digging for the next 8 steps.
/// Ants walking north avoid tunneling pheromone to ensure tunnels are always dug downward.
pub fn ants_add_tunnel_pheromone(
    mut ants_query: Query<(Entity, &Position, &AntOrientation), Without<Dead>>,
    pheromone_query: Query<&Pheromone>,
    pheromone_map: Res<PheromoneMap>,
    mut commands: Commands,
) {
    for (ant_entity, ant_position, ant_orientation) in ants_query.iter_mut() {
        if ant_orientation.is_facing_north() {
            continue;
        }

        if let Some(pheromone_entity) = pheromone_map.0.get(ant_position) {
            let pheromone = pheromone_query.get(*pheromone_entity).unwrap();

            match pheromone {
                Pheromone::Tunnel => {
                    commands.entity(ant_entity).insert(Tunneling(8));
                }
                _ => {}
            }
        }
    }
}

pub fn ants_remove_tunnel_pheromone(
    mut ants_query: Query<(Entity, Ref<Position>, &AntInventory, &mut Tunneling), Without<Dead>>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, position, inventory, mut tunneling) in ants_query.iter_mut() {
        if inventory.0 != None {
            // Ants lose tunneling when they start carrying anything.
            commands.entity(entity).remove::<Tunneling>();
            info!("Removed tunneling because ant is carrying something")
        } else if world_map.is_aboveground(position.as_ref()) {
            // Ants lose tunneling when they emerge on the surface.
            commands.entity(entity).remove::<Tunneling>();
            info!("Removed tunneling because ant is aboveground")
        } else if position.is_changed() {
            tunneling.0 -= 1;

            info!("Decremented tunneling to {}", tunneling.0);
            if tunneling.0 <= 0 {
                commands.entity(entity).remove::<Tunneling>();
                info!("Removed tunneling!");

                // If ant completed their tunneling pheromone naturally then it's time to build a chamber at the end of the tunnel.
                commands.spawn_pheromone(*position, Pheromone::Chamber);
            }
        }
    }
}
