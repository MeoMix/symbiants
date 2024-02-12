use bevy::prelude::*;

use crate::{
    common::{
        ant::{commands::AntCommandsExt, AntInventory, AntOrientation, Initiative},
        element::Element,
        grid::{Grid, GridElements},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

pub fn ants_dig(
    ants_query: Query<
        (
            &AntOrientation,
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
) {
    for (orientation, inventory, initiative, position, ant_entity) in ants_query.iter() {
        if !initiative.can_act() {
            continue;
        }

        // Consider digging / picking up the element under various circumstances.
        if inventory.0 != None {
            continue;
        }

        // TODO: Maybe support looking in 3 directions for food, but don't just pick one randomly here since it's weird it ignores food right in front of it.
        let position = orientation.get_ahead_position(position);

        if try_dig(
            ant_entity,
            position,
            &grid_query,
            &grid_elements,
            &mut commands,
        ) {
            return;
        }
    }
}

fn try_dig(
    ant_entity: Entity,
    dig_position: Position,
    grid_query: &Query<&Grid, With<AtCrater>>,
    grid_elements: &GridElements<AtCrater>,
    commands: &mut Commands,
) -> bool {
    let grid = grid_query.single();

    if !grid.is_within_bounds(&dig_position) {
        return false;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let element_entity = grid_elements.entity(dig_position);
    let element = grid_elements.element(*element_entity);

    if *element != Element::Food {
        return false;
    }

    commands.dig(ant_entity, dig_position, *element_entity, AtCrater);

    return true;
}
