use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    common::{
        ant::{commands::AntCommandsExt, initiative::Initiative, AntInventory},
        element::Element,
        grid::{Grid, GridElements},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

use super::CraterOrientation;

pub fn ants_dig(
    mut ants_query: Query<
        (
            &mut CraterOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            Entity,
        ),
        With<AtCrater>,
    >,
    grid_query: Query<&Grid, With<AtCrater>>,
    grid_elements: GridElements<AtCrater>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    for (mut orientation, inventory, initiative, position, ant_entity) in ants_query.iter_mut() {
        if !initiative.can_act() {
            continue;
        }

        // Consider digging / picking up the element under various circumstances.
        if inventory.0 != None {
            continue;
        }

        let grid = grid_query.single();

        let positions = vec![
            orientation.get_ahead_position(&position),
            orientation.get_clockwise_position(&position),
            orientation.get_counterclockwise_position(&position),
        ]
        .into_iter()
        .filter(|position| grid.is_within_bounds(position))
        .collect::<Vec<_>>();

        let food_positions = positions
            .iter()
            .filter_map(|&position| {
                let element_entity = grid_elements.entity(position);
                let element = grid_elements.element(*element_entity);

                match *element {
                    Element::Food => Some((position, *element_entity)),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        if food_positions.is_empty() {
            continue;
        }

        let (dig_position, dig_element_entity) = rng.sample(&food_positions).unwrap();
        commands.dig(ant_entity, *dig_position, *dig_element_entity, AtCrater);
        // TODO: This isn't right. I should express this as a separate system because `commands.dig` could fail
        *orientation = orientation.turn_around();
    }
}
