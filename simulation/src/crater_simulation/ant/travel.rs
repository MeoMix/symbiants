use bevy::prelude::*;

use crate::{
    common::{
        ant::{AntInventory, AntOrientation, Facing, Initiative},
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
            &mut AntOrientation,
            &Position,
            &AntInventory,
        ),
        With<AtCrater>,
    >,
    nest_query: Query<&Nest>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    let nest = nest_query.single();

    for (ant_entity, mut initiative, mut orientation, position, inventory) in ants_query.iter_mut()
    {
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

        // Leave the crater
        let mut ant_entity_commands = commands.entity(ant_entity);

        ant_entity_commands
            .remove::<AtCrater>()
            .insert(AtNest)
            .remove::<LeavingNest>()
            .remove::<LeavingFood>();

        // Make sure the ant is on its feet when it enters the nest
        // TODO: This is written in a hacky way where it uses commands to update AntOrientation because if AtNest/AtCrater is enqueued to be removed,
        // and orientation changes immediately, then on_update_ant_orientation runs against a stale zone and throws.
        if orientation.is_facing_north() {
            let rotated_orientation = orientation.rotate_forward();
            ant_entity_commands.insert(AntOrientation::new(rotated_orientation.get_facing(), rotated_orientation.get_angle()));
        } else if orientation.is_facing_south() {
            let rotated_orientation = orientation.rotate_backward();
            ant_entity_commands.insert(AntOrientation::new(rotated_orientation.get_facing(), rotated_orientation.get_angle()));
        }

        // TODO: There could be dirt/sand/food at the nest entrance - need to search and find a good place to put ant

        // If the ant enters the nest from the right-side entrance it should be placed on the right-side of the nest.
        // If the ant enters t he nest from the left-side entrance it should be placed on the left-side of the nest.
        if orientation.get_facing() == Facing::Left {
            ant_entity_commands.insert(Position::new(settings.nest_width, nest.surface_level()));
        } else {
            ant_entity_commands.insert(Position::new(0, nest.surface_level()));
        }

        initiative.consume();
    }
}
