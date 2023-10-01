use bevy::{prelude::*, utils::HashMap};
use bevy_save::SaveableRegistry;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

pub mod commands;
pub mod ui;

use crate::{
    ant::{
        commands::AntCommandsExt, walk::get_turned_orientation, AntInventory, AntOrientation, Dead,
        Initiative,
    },
    common::register,
    element::Element,
    pheromone::commands::PheromoneCommandsExt,
    world_map::{position::Position, WorldMap},
};

// TODO: better home for all the pheromone stuff.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
pub enum Pheromone {
    #[default]
    Tunnel,
    Chamber,
}

/// Note the intentional omission of reflection/serialization.
/// This is because PheromoneMap is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Resource, Debug)]
pub struct PheromoneMap(pub HashMap<Position, Entity>);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Tunneling(pub isize);

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Chambering(pub isize);

/// Registers Pheromone-related Resource and Component types with bevy for persistence.
/// Note the intentional omission of HashMap<Position, Pheromone>. It would be wasteful to persist
/// because it's able to be trivially regenerated at runtime.
pub fn register_pheromone(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Tunneling>(&app_type_registry, &mut saveable_registry);
    register::<Chambering>(&app_type_registry, &mut saveable_registry);
    register::<Pheromone>(&app_type_registry, &mut saveable_registry);
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to pheromone entities for quick lookup outside of ECS architecture.
///
/// This isn't super necessary. Performance impact of O(N) lookup on Pheromone is likely to be negligible.
/// Still, it seemed like a good idea architecturally to have O(1) lookup when Position is known.
pub fn setup_pheromone(
    pheromone_query: Query<(&mut Position, Entity), With<Pheromone>>,
    mut commands: Commands,
) {
    let pheromone_map = pheromone_query
        .iter()
        .map(|(position, entity)| (*position, entity))
        .collect::<HashMap<_, _>>();

    commands.insert_resource(PheromoneMap(pheromone_map));
}

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

            info!("ants_tunnel_pheromone_move - consumed movement");

            initiative.consume_movement();
            continue;
        }

        // Blocked, defer to default action
        if !has_air_ahead && !has_air_above {
            info!("ants_tunnel_pheromone_move - blocked");
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

            info!("ants_tunnel_pheromone_move - consumed movement 2");
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

        info!("digging ahead");

        initiative.consume_action();
    }
}

pub fn ants_tunnel_pheromone(
    mut ants_query: Query<
        (Entity, Ref<Position>, &AntInventory, Option<&mut Tunneling>),
        Without<Dead>,
    >,
    pheromone_query: Query<&Pheromone>,
    pheromone_map: Res<PheromoneMap>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, position, inventory, tunneling) in ants_query.iter_mut() {
        if inventory.0 != None && tunneling.is_some() {
            // Ants lose tunneling when they start carrying anything.
            commands.entity(entity).remove::<Tunneling>();
            info!("Removed tunneling because ant is carrying something")
        } else if world_map.is_aboveground(position.as_ref()) && tunneling.is_some() {
            // Ants lose tunneling when they emerge on the surface.
            commands.entity(entity).remove::<Tunneling>();
            info!("Removed tunneling because ant is aboveground")
        } else if position.is_changed() {
            if let Some(mut tunneling) = tunneling {
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

    // Whenever an ant walks over a tile which has a pheromone, it will gain a Component representing that Pheromone.
    for (ant_entity, ant_position, _, tunneling) in ants_query.iter_mut() {
        if let Some(pheromone_entity) = pheromone_map.0.get(ant_position.as_ref()) {
            let pheromone = pheromone_query.get(*pheromone_entity).unwrap();

            match pheromone {
                Pheromone::Tunnel => {
                    if let Some(mut tunneling) = tunneling {
                        tunneling.0 = 8;
                        info!("Reset tunneling to 8");
                    } else {
                        commands.entity(ant_entity).insert(Tunneling(8));
                        info!("Set tunneling to 8!");
                    }
                }
                _ => {}
            }
        }
    }
}

// Act on Chamber pheromone
pub fn ants_chamber_pheromone_act(
    mut ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
            &Position,
            Entity,
        ),
        (Without<Dead>, With<Chambering>),
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
) {
    // If covered in Chamber pheromone then the following things need to occur:
    //  1) Look forward in the direction the ant is facing - if something is diggable - dig it.
    //  2) Look up in the direction in the ant is facing - if something is diggable - dig it.
    //  3) Either step forward or turn around
    //  4) Repeat while covered in pheromone

    for (orientation, inventory, mut initiative, ant_position, ant_entity) in ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        // Safeguard, but not expected to run because shouldn't have Chambering pheromone with full inventory.
        if inventory.0 != None {
            continue;
        }

        info!("Chambering!");

        if try_dig(
            &ant_entity,
            &orientation.get_ahead_position(ant_position),
            &elements_query,
            &world_map,
            &mut commands,
        ) {
            initiative.consume_action();
            continue;
        }

        if try_dig(
            &ant_entity,
            &orientation.get_above_position(ant_position),
            &elements_query,
            &world_map,
            &mut commands,
        ) {
            initiative.consume_action();
            continue;
        }

        // TODO: maybe take control of walking here? idk.
    }
}

fn try_dig(
    ant_entity: &Entity,
    dig_position: &Position,
    elements_query: &Query<&Element>,
    world_map: &WorldMap,
    commands: &mut Commands,
) -> bool {
    if !world_map.is_within_bounds(&dig_position) {
        return false;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let element_entity = world_map.get_element(*dig_position).unwrap();
    let Ok(element) = elements_query.get(*element_entity) else {
        panic!("act - expected entity to exist")
    };

    if *element == Element::Air {
        return false;
    }

    commands.dig(*ant_entity, *dig_position, *element_entity);

    true
}

pub fn ants_chamber_pheromone(
    mut ants_query: Query<
        (
            Entity,
            Ref<Position>,
            &AntInventory,
            Option<&mut Chambering>,
        ),
        Without<Dead>,
    >,
    pheromone_query: Query<&Pheromone>,
    pheromone_map: Res<PheromoneMap>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, position, inventory, chambering) in ants_query.iter_mut() {
        // Ant gets Chamber pheromone on it, Chamber pheromone decrements by 1 each step it takes
        // While covered in Chamber pheromone, ant will aggressively dig at anything it sees that is at-or-(above?) it.
        // Ants need the ability to dig diagonally (?) or they need to not always dig the same direction when covered in pheromone.
        if inventory.0 != None && chambering.is_some() {
            // Ants lose chambering when they start carrying anything.
            commands.entity(entity).remove::<Chambering>();
            info!("Removed chambering because ant is carrying something")
        } else if world_map.is_aboveground(position.as_ref()) && chambering.is_some() {
            // Ants lose chambering when they emerge on the surface.
            commands.entity(entity).remove::<Chambering>();
            info!("Removed chambering because ant is aboveground")
        } else if position.is_changed() {
            if let Some(mut chambering) = chambering {
                chambering.0 -= 1;
                info!("Decremented chambering to {}", chambering.0);
                if chambering.0 <= 0 {
                    commands.entity(entity).remove::<Chambering>();
                    info!("Removed chambering!");
                }
            }
        }
    }

    // Whenever an ant walks over a tile which has a pheromone, it will gain a Component representing that Pheromone.
    for (ant_entity, ant_position, _, chambering) in ants_query.iter_mut() {
        if let Some(pheromone_entity) = pheromone_map.0.get(ant_position.as_ref()) {
            let pheromone = pheromone_query.get(*pheromone_entity).unwrap();

            match pheromone {
                Pheromone::Chamber => {
                    if let Some(mut chambering) = chambering {
                        chambering.0 = 2;
                        info!("Reset chambering to 2");
                    } else {
                        commands.entity(ant_entity).insert(Chambering(8));
                        info!("Set chambering to 2!");
                    }
                }
                _ => {}
            }
        }
    }
}
