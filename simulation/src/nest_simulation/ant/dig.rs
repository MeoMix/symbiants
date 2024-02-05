use super::{commands::AntCommandsExt, AntInventory, AntOrientation, AntRole, Initiative};
use crate::{
    common::{
        grid::{Grid, GridElements},
        position::Position,
    },
    nest_simulation::{
        element::Element,
        nest::{AtNest, Nest},
    },
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub fn ants_dig(
    ants_query: Query<
        (
            &AntOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            &AntRole,
            Entity,
        ),
        With<AtNest>,
    >,
    nest_query: Query<(&Grid, &Nest)>,
    grid_elements: GridElements<AtNest>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    for (orientation, inventory, initiative, position, role, ant_entity) in ants_query.iter() {
        if !initiative.can_act() {
            continue;
        }

        // Consider digging / picking up the element under various circumstances.
        if inventory.0 != None {
            continue;
        }

        let positions = [
            orientation.get_ahead_position(position),
            orientation.get_below_position(position),
            orientation.get_above_position(position),
        ];

        let position = rng.sample(&positions).unwrap();

        if try_dig(
            ant_entity,
            role,
            *position,
            &ants_query,
            &nest_query,
            &grid_elements,
            &mut commands,
            &settings,
            &mut rng,
        ) {
            return;
        }
    }
}

fn try_dig(
    ant_entity: Entity,
    ant_role: &AntRole,
    dig_position: Position,
    ants_query: &Query<
        (
            &AntOrientation,
            &AntInventory,
            &Initiative,
            &Position,
            &AntRole,
            Entity,
        ),
        With<AtNest>,
    >,
    nest_query: &Query<(&Grid, &Nest)>,
    grid_elements: &GridElements<AtNest>,
    commands: &mut Commands,
    settings: &Res<Settings>,
    rng: &mut ResMut<GlobalRng>,
) -> bool {
    let (grid, nest) = nest_query.single();

    if !grid.is_within_bounds(&dig_position) {
        return false;
    }

    // Check if hitting a solid element and, if so, consider digging through it.
    let element_entity = grid_elements.entity(dig_position);
    let element = grid_elements.element(*element_entity);
    if *element == Element::Air {
        return false;
    }

    // NOTE: can remove this in the future when adding more elements
    if *element != Element::Sand && *element != Element::Food {
        return false;
    }

    // For workers, check if digging near queen and if so prioritize it because it's immersion breaking
    // seeing stuff stacked on the queen and her not moving to respond to it.
    if *ant_role == AntRole::Worker {
        let adjacent_queen = ants_query.iter().find(|(_, _, _, position, &role, _)| {
            role == AntRole::Queen && dig_position.distance(position) <= 1
        });

        if adjacent_queen.is_some() {
            commands.dig(ant_entity, dig_position, *element_entity, AtNest);

            return true;
        }
    }

    let mut dig = false;

    if *element == Element::Food && *ant_role == AntRole::Worker {
        // When above ground, workers prioritize picking up food. Queen needs to focus on nest construction.
        if nest.is_aboveground(&dig_position) {
            dig = rng.f32() < settings.probabilities.above_surface_food_dig;
        } else {
            dig = rng.f32() < settings.probabilities.below_surface_food_dig;
        }
    } else if *element == Element::Sand && nest.is_underground(&dig_position) {
        // When underground, prioritize clearing out sand and allow for digging tunnels through dirt. Leave food underground.
        // It's OK for queen to pick up sand because sometimes it'll get in the way of nest building.
        dig = *element == Element::Sand && nest.is_underground(&dig_position);
    }

    if dig {
        commands.dig(ant_entity, dig_position, *element_entity, AtNest);

        return true;
    }

    return false;
}
