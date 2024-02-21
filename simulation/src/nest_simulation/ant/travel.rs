use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{
        ant::{AntInventory, AntOrientation, AntRole, Initiative},
        grid::Grid,
        position::Position,
    },
    crater_simulation::{ant::emit_pheromone::LeavingNest, crater::AtCrater},
    nest_simulation::nest::{AtNest, Nest},
    settings::Settings,
};

use super::{chambering::Chambering, tunneling::Tunneling};

// TODO: Maybe put this in common since it relies on knowledge of AtCrater and AtNest

/// If an ant is on the surface, and it's facing the edge of the nest, and it's not carrying anything
/// then it is able to leave the nest and go out into the crater.
pub fn ants_travel_to_crater(
    mut ants_query: Query<
        (
            Entity,
            &mut Initiative,
            &Position,
            &AntOrientation,
            &AntInventory,
            &AntRole,
        ),
        With<AtNest>,
    >,
    nest_query: Query<(&Grid, &Nest), With<AtNest>>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    let (grid, nest) = nest_query.single();

    for (ant_entity, mut initiative, position, orientation, inventory, role) in
        ants_query.iter_mut()
    {
        if !initiative.can_move() {
            continue;
        }

        if *role == AntRole::Queen {
            continue;
        }

        if inventory.0 != None {
            continue;
        }

        if !orientation.is_rightside_up() {
            continue;
        }

        if !nest.is_aboveground(position) {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(&position);
        if grid.is_within_bounds(&ahead_position) {
            continue;
        }

        // TODO: Adjust probability and read from settings
        // Must be attempting to walk outside the bounds of the nest - consider leaving
        if rng.chance(0.5) {
            continue;
        }

        // TODO: Consider despawning + spawning entirely rather than trying to micromanage removal of components
        // Leave the nest
        commands
            .entity(ant_entity)
            .remove::<AtNest>()
            .remove::<Tunneling>()
            .remove::<Chambering>()
            .insert(AtCrater)
            .insert(LeavingNest(1000.0))
            .insert(Position::new(
                // TODO: Express this more clearly - trying to not have it appear ontop of the nest sprite
                (settings.crater_width / 2) + 1,
                (settings.crater_height / 2) + 1,
            ));

        initiative.consume();
    }
}
