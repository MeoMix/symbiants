use super::birthing::Birthing;
use crate::{
    common::{
        ant::{commands::AntCommandsExt, AntInventory, NestOrientation, Initiative},
        element::Element,
        grid::{Grid, GridElements},
        pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneMap, PheromoneStrength},
        position::Position,
    },
    nest_simulation::{
        ant::walk::get_turned_orientation,
        nest::{AtNest, Nest},
    },
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

// TODO: Consider replacing this with something more like a "CurrentAction" enum to reflect that an Ant shouldn't perform multiple actions at once.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Tunneling(pub f32);

// "Whenever ant walks over tile with nesting pheromone, they gain "Nesting: 8". Then, they attempt to take a step forward and decrement Nesting to 7. If they end up digging, nesting is "forgotten" and they shift back to hauling dirt
// so they haul it out, then walk back, hit the pheromone again, and repeatedly try to take 8 steps
// and if they succeed in getting to nesting 0, then drop a new pheromone marker "build chamber here"
// and when they hit that marker, they will look around them in all directions and, if they see any adjacent dirt, they will dig one piece of it and go back into "haul dirt" mode
// if they do not see any adjacent dirt, they clean up the pheromone and, in the case of the queen, shift to giving birth
pub fn ants_tunnel_pheromone_move(
    mut ants_query: Query<
        (&mut NestOrientation, &mut Initiative, &mut Position),
        (With<Tunneling>, With<AtNest>),
    >,
    nest_query: Query<&Nest>,
    mut rng: ResMut<GlobalRng>,
    grid_elements: GridElements<AtNest>,
) {
    let nest = nest_query.single();

    for (mut orientation, mut initiative, mut ant_position) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // Ants that are following a tunneling pheromone will prioritize walking into open space in front of them
        // over digging directly in front of them.

        // If there's solid material in front of ant then consider turning onto it if there's tunnel to follow upward.
        let ahead_position = orientation.get_ahead_position(&ant_position);
        let has_air_ahead = grid_elements
            .get_entity(ahead_position)
            .map_or(false, |entity| {
                grid_elements
                    .get_element(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        let above_position = orientation.get_above_position(&ant_position);
        let has_air_above = grid_elements
            .get_entity(above_position)
            .map_or(false, |entity| {
                grid_elements
                    .get_element(*entity)
                    .map_or(false, |element| *element == Element::Air)
            });

        if !has_air_ahead && has_air_above {
            *orientation = get_turned_orientation(
                &orientation,
                &ant_position,
                &nest,
                &mut rng,
                &grid_elements,
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

        if let Some(foot_entity) = grid_elements.get_entity(foot_position) {
            let foot_element = grid_elements.element(*foot_entity);

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
    ants_query: Query<
        (
            &NestOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            Entity,
            &Tunneling,
        ),
        With<AtNest>,
    >,
    grid_query: Query<&Grid, With<AtNest>>,
    grid_elements: GridElements<AtNest>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    let grid = grid_query.single();

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
        if !grid.is_within_bounds(&ahead_position) {
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = grid_elements.entity(ahead_position);
        let element = grid_elements.element(*entity);

        if *element == Element::Air {
            continue;
        }

        let dig_position = orientation.get_ahead_position(position);
        let dig_target_entity = *grid_elements.entity(dig_position);
        commands.dig(ant_entity, dig_position, dig_target_entity, AtNest);

        // Reduce PheromoneStrength by 1 because not digging at ant_position, but ant_position + 1.
        // If this didn't occur then either the ant would need to apply strength-1 to itself when stepping onto a tile, or
        // PheromoneStrength would never reduce.
        if tunneling.0 - 1.0 > 0.0 {
            commands.spawn_pheromone(
                dig_position,
                Pheromone::Tunnel,
                PheromoneStrength::new(tunneling.0 - 1.0, settings.tunnel_length as f32),
                AtNest,
            );
        }
    }
}

/// Apply tunneling to ants which walk over tiles covered in tunnel pheromone.
/// Tunneling is set to Tunneling(8). This encourages ants to prioritize digging for the next 8 steps.
/// Ants walking north avoid tunneling pheromone to ensure tunnels are always dug downward.
pub fn ants_add_tunnel_pheromone(
    ants_query: Query<
        (Entity, &Position, &AntInventory, &NestOrientation),
        (
            Changed<Position>,
            With<Initiative>,
            Without<Birthing>,
            With<AtNest>,
        ),
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength)>,
    pheromone_map: Res<PheromoneMap<AtNest>>,
    mut commands: Commands,
) {
    for (ant_entity, ant_position, inventory, ant_orientation) in ants_query.iter() {
        if inventory.0 != None {
            continue;
        }

        if ant_orientation.is_facing_north() {
            continue;
        }

        if let Some(pheromone_entities) = pheromone_map.get(ant_position) {
            // There should only be one Pheromone::Tunnel at a given position.
            for pheromone_entity in pheromone_entities {
                let (pheromone, pheromone_strength) =
                    pheromone_query.get(*pheromone_entity).unwrap();

                if *pheromone == Pheromone::Tunnel {
                    commands
                        .entity(ant_entity)
                        .insert(Tunneling(pheromone_strength.value()));
                }
            }
        }
    }
}

/// Whenever an ant takes a step it loses 1 Tunneling pheromone.
pub fn ants_fade_tunnel_pheromone(
    mut ants_query: Query<&mut Tunneling, (Changed<Position>, With<AtNest>)>,
) {
    for mut tunneling in ants_query.iter_mut() {
        tunneling.0 -= 1.0;
    }
}

/// Ants lose Tunneling when they begin carrying anything because they've fulfilled the pheromones action.
/// Ants lose Tunneling when they emerge on the surface because tunnels aren't dug aboveground.
/// Ants lose Tunneling when they've exhausted their pheromone by taking sufficient steps.
pub fn ants_remove_tunnel_pheromone(
    mut ants_query: Query<
        (Entity, &Position, &AntInventory, &Tunneling),
        (Or<(Changed<Position>, Changed<AntInventory>)>, With<AtNest>),
    >,
    pheromone_query: Query<(&Pheromone, &PheromoneStrength)>,
    pheromone_map: Res<PheromoneMap<AtNest>>,
    grid_elements: GridElements<AtNest>,
    mut commands: Commands,
    nest_query: Query<&Nest>,
    settings: Res<Settings>,
) {
    let nest = nest_query.single();

    for (ant_entity, ant_position, inventory, tunneling) in ants_query.iter_mut() {
        if inventory.0 != None {
            commands.entity(ant_entity).remove::<Tunneling>();
        } else if nest.is_aboveground(ant_position) {
            commands.entity(ant_entity).remove::<Tunneling>();
        } else if tunneling.0 <= 0.0 {
            commands.entity(ant_entity).remove::<Tunneling>();

            let adjacent_positions = ant_position.get_adjacent_positions();

            let adjacent_pheromones = adjacent_positions
                .iter()
                .filter_map(|position| pheromone_map.get(position))
                .flat_map(|entities| entities.iter())
                .filter_map(|entity| pheromone_query.get(*entity).ok())
                .collect::<Vec<(&Pheromone, &PheromoneStrength)>>();

            let has_adjacant_low_strength_tunnel_pheromone =
                adjacent_pheromones
                    .iter()
                    .any(|(&pheromone, &pheromone_strength)| {
                        // TODO: not sure this logic still makes sense after switching to float
                        pheromone == Pheromone::Tunnel && pheromone_strength.value() <= 1.0
                    });

            // Confirm that ant is at the end of a tunnel by checking that there is only air on one side of it
            // Otherwise, might be in the middle of a tunnel with an expiring pheromone trail.
            let adjacent_air_positions = adjacent_positions
                .iter()
                .filter(|&position| {
                    if let Some(element_entity) = grid_elements.get_entity(*position) {
                        return *grid_elements.element(*element_entity) == Element::Air;
                    }

                    false
                })
                .collect::<Vec<_>>();

            if has_adjacant_low_strength_tunnel_pheromone && adjacent_air_positions.len() <= 1 {
                // If ant completed their tunneling pheromone naturally then it's time to build a chamber at the end of the tunnel.
                commands.spawn_pheromone(
                    *ant_position,
                    Pheromone::Chamber,
                    PheromoneStrength::new(
                        settings.chamber_size as f32,
                        settings.chamber_size as f32,
                    ),
                    AtNest,
                );
            }
        }
    }
}
