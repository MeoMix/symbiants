use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    ant::{AntInventory, AntOrientation, Dead, Initiative, commands::AntCommandsExt},
    element::Element,
    world_map::{position::Position, WorldMap}, pheromone::{Pheromone, PheromoneMap},
};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Chambering(pub isize);

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

// TODO: better home for this? maybe in commands?
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
