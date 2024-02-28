use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{
        ant::{
            AntInventory, CraterOrientation, Initiative, NestAngle, NestFacing, NestOrientation,
        },
        position::Position,
    },
    crater_simulation::{ant::emit_pheromone::LeavingNest, crater::AtCrater},
    nest_simulation::nest::{AtNest, Nest},
    settings::Settings,
};

use super::emit_pheromone::LeavingFood;

// TODO: Maybe put this in common since it relies on knowledge of AtCrater and AtNest

/// If an ant walks into the "Nest Entrance" area of the crater then it is able to enter into the nest
/// TODO: Need to more intelligently find a place to put the ant rather than spawning into potential dirt.
pub fn ants_travel_to_nest(
    mut ants_query: Query<
        (
            Entity,
            &mut Initiative,
            &CraterOrientation,
            &Position,
            &AntInventory,
        ),
        With<AtCrater>,
    >,
    nest_query: Query<&Nest>,
    mut commands: Commands,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
) {
    let nest = nest_query.single();

    for (ant_entity, mut initiative, orientation, position, inventory) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        // TODO: Instead of only allowing ants to return home when they have food - allow them to return home when they are low on energy/have given up on their task
        if inventory.0.is_none() {
            continue;
        }

        let nest_position = Position::new(settings.crater_width / 2, settings.crater_height / 2);
        if Position::distance(position, &nest_position) > 1 {
            continue;
        }

        let mut inventory_element_commands = commands.entity(inventory.0.unwrap());
        inventory_element_commands.remove::<AtCrater>().insert(AtNest);

        // Leave the crater
        let mut ant_entity_commands = commands.entity(ant_entity);

        ant_entity_commands
            .remove::<AtCrater>()
            .remove::<CraterOrientation>()
            .insert(AtNest)
            .remove::<LeavingNest>()
            .remove::<LeavingFood>();

        // TODO: There could be dirt/sand/food at the nest entrance - need to search and find a good place to put ant
        // Make sure the ant is on its feet when it enters the nest
        // If the ant enters the nest from the right-side entrance it should be placed on the right-side of the nest.
        // If the ant enters t he nest from the left-side entrance it should be placed on the left-side of the nest.
        if *orientation == CraterOrientation::Left {
            ant_entity_commands.insert(Position::new(settings.nest_width, nest.surface_level()));
            ant_entity_commands.insert(NestOrientation::new(NestFacing::Left, NestAngle::Zero));
        } else if *orientation == CraterOrientation::Right {
            ant_entity_commands.insert(Position::new(0, nest.surface_level()));
            ant_entity_commands.insert(NestOrientation::new(NestFacing::Right, NestAngle::Zero));
        } else {
            // Choose left or right entrance at random
            if rng.bool() {
                ant_entity_commands
                    .insert(Position::new(settings.nest_width, nest.surface_level()));
                ant_entity_commands.insert(NestOrientation::new(NestFacing::Left, NestAngle::Zero));
            } else {
                ant_entity_commands.insert(Position::new(0, nest.surface_level()));
                ant_entity_commands
                    .insert(NestOrientation::new(NestFacing::Right, NestAngle::Zero));
            }
        }

        initiative.consume();
    }
}
