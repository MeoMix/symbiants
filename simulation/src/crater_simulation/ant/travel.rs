use bevy::prelude::*;

use crate::{
    common::{
        ant::{AntOrientation, Initiative},
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
    mut ants_query: Query<(Entity, &mut Initiative, &Position, &AntOrientation), With<AtCrater>>,
    nest_query: Query<&Nest>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    let nest = nest_query.single();

    for (ant_entity, mut initiative, position, orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let center = Position::new(settings.crater_width / 2, settings.crater_height / 2);
        let ahead_position = orientation.get_ahead_position(&position);
        if ahead_position != center {
            continue;
        }

        // Leave the crater
        commands
            .entity(ant_entity)
            .remove::<AtCrater>()
            .insert(AtNest)
            .remove::<LeavingNest>()
            .remove::<LeavingFood>()
            .insert(Position::new(0, nest.surface_level()));

        initiative.consume();
    }
}
