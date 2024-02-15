use super::birthing::Birthing;
use crate::{
    common::{
        ant::{commands::AntCommandsExt, AntInventory, AntOrientation, Initiative},
        element::Element,
        grid::{Grid, GridElements},
        pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneMap, PheromoneStrength},
        position::Position,
    },
    nest_simulation::nest::{AtNest, Nest},
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Chambering(pub isize);

/// If covered in Chamber pheromone then the following things need to occur:
///  1) Look forward in the direction the ant is facing - if something is diggable - dig it.
///  2) Look up in the direction in the ant is facing - if something is diggable - dig it.
///  3) Either step forward or turn around
///  4) Repeat while covered in pheromone
pub fn ants_chamber_pheromone_act(
    ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            Entity,
            &Chambering,
        ),
        With<AtNest>,
    >,
    grid_query: Query<&Grid, With<AtNest>>,
    grid_elements: GridElements<AtNest>,
    mut commands: Commands,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
) {
    for (ant_orientation, inventory, initiative, ant_position, ant_entity, chambering) in
        ants_query.iter()
    {
        if !initiative.can_act() {
            continue;
        }

        // Safeguard, but not expected to run because shouldn't have Chambering pheromone with full inventory.
        if inventory.0 != None {
            continue;
        }

        let positions = vec![
            ant_orientation.get_ahead_position(ant_position),
            ant_orientation.get_below_position(ant_position),
            ant_orientation.get_above_position(ant_position),
        ];

        let position = rng.sample(&positions).unwrap();

        if try_dig(
            &ant_entity,
            &position,
            &grid_query,
            &grid_elements,
            &mut commands,
        ) {
            // Subtract 1 because not placing pheromone at ant_position but instead placing it at a position adjacent
            if chambering.0 - 1 > 0 {
                commands.spawn_pheromone(
                    *position,
                    Pheromone::Chamber,
                    PheromoneStrength::new(chambering.0 - 1, settings.chamber_size),
                    AtNest,
                );
            }
            return;
        }
    }
}

/// Apply chambering to ants which walk over tiles covered in chamber pheromone.
/// Chambering is set to Chambering(3). This encourages ants to dig for the next 3 steps.
pub fn ants_add_chamber_pheromone(
    ants_query: Query<
        (Entity, &Position, &AntInventory),
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
    for (ant_entity, ant_position, inventory) in ants_query.iter() {
        if inventory.0 != None {
            continue;
        }

        if let Some(pheromone_entities) = pheromone_map.map.get(ant_position) {
            // There should only be one Pheromone::Chamber at a given position.
            for pheromone_entity in pheromone_entities {
                let (pheromone, pheromone_strength) =
                    pheromone_query.get(*pheromone_entity).unwrap();

                if *pheromone == Pheromone::Chamber {
                    commands
                        .entity(ant_entity)
                        .insert(Chambering(pheromone_strength.value()));
                }
            }
        }
    }
}

/// Whenever an ant takes a step it loses 1 Chambering pheromone.
pub fn ants_fade_chamber_pheromone(mut ants_query: Query<&mut Chambering, Changed<Position>>) {
    for mut chambering in ants_query.iter_mut() {
        chambering.0 -= 1;
    }
}

/// Ants lose Chambering when they begin carrying anything because they've fulfilled the pheromones action.
/// Ants lose Chambering when they emerge on the surface because chambers aren't dug aboveground.
/// Ants lose Chambering when they've exhausted their pheromone by taking sufficient steps.
pub fn ants_remove_chamber_pheromone(
    mut ants_query: Query<
        (Entity, &Position, &AntInventory, &Chambering),
        (Or<(Changed<Position>, Changed<AntInventory>)>, With<AtNest>),
    >,
    mut commands: Commands,
    nest_query: Query<&Nest>,
) {
    let nest = nest_query.single();

    for (entity, position, inventory, chambering) in ants_query.iter_mut() {
        if inventory.0 != None {
            commands.entity(entity).remove::<Chambering>();
        } else if nest.is_aboveground(position) {
            commands.entity(entity).remove::<Chambering>();
        } else if chambering.0 <= 0 {
            commands.entity(entity).remove::<Chambering>();
        }
    }
}

fn try_dig(
    ant_entity: &Entity,
    dig_position: &Position,
    grid_query: &Query<&Grid, With<AtNest>>,
    grid_elements: &GridElements<AtNest>,
    commands: &mut Commands,
) -> bool {
    if !grid_query.single().is_within_bounds(&dig_position) {
        return false;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let element_entity = grid_elements.entity(*dig_position);
    let element = grid_elements.element(*element_entity);
    if *element == Element::Air {
        return false;
    }

    commands.dig(*ant_entity, *dig_position, *element_entity, AtNest);

    true
}
