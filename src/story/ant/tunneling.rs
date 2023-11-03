use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use crate::{story::{
    ant::{
        commands::AntCommandsExt, walk::get_turned_orientation, AntInventory, AntOrientation,
        Initiative,
    },
    common::position::Position,
    element::Element,
    nest_simulation::nest::Nest,
    pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneMap, PheromoneStrength},

}, settings::Settings};

use super::birthing::Birthing;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Tunneling(pub isize);

// "Whenever ant walks over tile with nesting pheromone, they gain "Nesting: 8". Then, they attempt to take a step forward and decrement Nesting to 7. If they end up digging, nesting is "forgotten" and they shift back to hauling dirt
// so they haul it out, then walk back, hit the pheromone again, and repeatedly try to take 8 steps
// and if they succeed in getting to nesting 0, then drop a new pheromone marker "build chamber here"
// and when they hit that marker, they will look around them in all directions and, if they see any adjacent dirt, they will dig one piece of it and go back into "haul dirt" mode
// if they do not see any adjacent dirt, they clean up the pheromone and, in the case of the queen, shift to giving birth
pub fn ants_tunnel_pheromone_move(
    mut ants_query: Query<(&mut AntOrientation, &mut Initiative, &mut Position), With<Tunneling>>,
    elements_query: Query<&Element>,
    nest: Res<Nest>,
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
        let has_air_ahead = nest
            .get_element_entity(ahead_position)
            .map_or(false, |entity| {
                elements_query
                    .get(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        let above_position = orientation.get_above_position(&ant_position);
        let has_air_above = nest
            .get_element_entity(above_position)
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
                &nest,
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

        if let Some(foot_entity) = nest.get_element_entity(foot_position) {
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
    ants_query: Query<(
        &AntOrientation,
        &AntInventory,
        &Initiative,
        &Position,
        Entity,
        &Tunneling,
    )>,
    elements_query: Query<&Element>,
    nest: Res<Nest>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    for (orientation, inventory, initiative, position, ant_entity, tunneling) in ants_query.iter() {
        if !initiative.can_act() {
            continue;
        }

        // Safeguard, but not expected to run because shouldn't have Tunneling pheromone with full inventory.
        if inventory.0 != None {
            continue;
        }

        // Tunneling up only results in breaching the surface which isn't desirable because then sand pours
        // in constantly and overwhelms the colony.
        if orientation.is_facing_north() {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(position);
        if !nest.is_within_bounds(&ahead_position) {
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = nest.element_entity(ahead_position);
        let Ok(element) = elements_query.get(*entity) else {
            panic!("act - expected entity to exist")
        };

        if *element == Element::Air {
            continue;
        }

        let dig_position = orientation.get_ahead_position(position);
        let dig_target_entity = *nest.element_entity(dig_position);
        commands.dig(ant_entity, dig_position, dig_target_entity);

        // Reduce PheromoneStrength by 1 because not digging at ant_position, but ant_position + 1.
        // If this didn't occur then either the ant would need to apply strength-1 to itself when stepping onto a tile, or
        // PheromoneStrength would never reduce.
        if tunneling.0 - 1 > 0 {
            commands.spawn_pheromone(
                dig_position,
                Pheromone::Tunnel,
                PheromoneStrength::new(tunneling.0 - 1, settings.tunnel_length),
            );
        }
    }
}

/// Apply tunneling to ants which walk over tiles covered in tunnel pheromone.
/// Tunneling is set to Tunneling(8). This encourages ants to prioritize digging for the next 8 steps.
/// Ants walking north avoid tunneling pheromone to ensure tunnels are always dug downward.
pub fn ants_add_tunnel_pheromone(
    ants_query: Query<
        (Entity, &Position, &AntInventory, &AntOrientation),
        (Changed<Position>, With<Initiative>, Without<Birthing>),
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength)>,
    pheromone_map: Res<PheromoneMap>,
    mut commands: Commands,
) {
    for (ant_entity, ant_position, inventory, ant_orientation) in ants_query.iter() {
        if inventory.0 != None {
            continue;
        }

        if ant_orientation.is_facing_north() {
            continue;
        }

        if let Some(pheromone_entity) = pheromone_map.0.get(ant_position) {
            let (pheromone, pheromone_strength) = pheromone_query.get(*pheromone_entity).unwrap();

            if *pheromone == Pheromone::Tunnel {
                commands
                    .entity(ant_entity)
                    .insert(Tunneling(pheromone_strength.value()));
            }
        }
    }
}

/// Whenever an ant takes a step it loses 1 Tunneling pheromone.
pub fn ants_fade_tunnel_pheromone(mut ants_query: Query<&mut Tunneling, Changed<Position>>) {
    for mut tunneling in ants_query.iter_mut() {
        tunneling.0 -= 1;
    }
}

/// Ants lose Tunneling when they begin carrying anything because they've fulfilled the pheromones action.
/// Ants lose Tunneling when they emerge on the surface because tunnels aren't dug aboveground.
/// Ants lose Tunneling when they've exhausted their pheromone by taking sufficient steps.
pub fn ants_remove_tunnel_pheromone(
    mut ants_query: Query<
        (Entity, &Position, &AntInventory, &Tunneling),
        Or<(Changed<Position>, Changed<AntInventory>)>,
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength)>,
    pheromone_map: Res<PheromoneMap>,
    mut commands: Commands,
    nest: Res<Nest>,
    settings: Res<Settings>,
) {
    for (ant_entity, ant_position, inventory, tunneling) in ants_query.iter_mut() {
        if inventory.0 != None {
            commands.entity(ant_entity).remove::<Tunneling>();
        } else if nest.is_aboveground(ant_position) {
            commands.entity(ant_entity).remove::<Tunneling>();
        } else if tunneling.0 <= 0 {
            commands.entity(ant_entity).remove::<Tunneling>();

            let adjacent_pheromones = ant_position
                .get_adjacent_positions()
                .iter()
                .filter_map(|position| {
                    pheromone_map
                        .0
                        .get(position)
                        .and_then(|pheromone_entity| pheromone_query.get(*pheromone_entity).ok())
                })
                .collect::<Vec<(&Pheromone, &PheromoneStrength)>>();

            let has_adjacant_low_strength_tunnel_pheromone =
                adjacent_pheromones
                    .iter()
                    .any(|(&pheromone, &pheromone_strength)| {
                        pheromone == Pheromone::Tunnel && pheromone_strength.value() == 1
                    });

            if has_adjacant_low_strength_tunnel_pheromone {
                // If ant completed their tunneling pheromone naturally then it's time to build a chamber at the end of the tunnel.
                commands.spawn_pheromone(
                    *ant_position,
                    Pheromone::Chamber,
                    PheromoneStrength::new(settings.chamber_size, settings.chamber_size),
                );
            }
        }
    }
}
