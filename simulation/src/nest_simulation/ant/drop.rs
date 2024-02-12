use super::{commands::AntCommandsExt, AntInventory, AntOrientation, AntRole, Initiative};
use crate::{
    common::{
        element::Element,
        grid::{Grid, GridElements},
        position::Position,
    },
    nest_simulation::nest::{AtNest, Nest},
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub fn ants_drop(
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
    elements_query: Query<&Element>,
    nest_query: Query<(&Grid, &Nest)>,
    grid_elements: GridElements<AtNest>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let (grid, nest) = nest_query.single();

    for (orientation, inventory, initiative, position, role, ant_entity) in ants_query.iter() {
        if !initiative.can_act() {
            continue;
        }

        if inventory.0 == None {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(position);
        if !grid.is_within_bounds(&ahead_position) {
            continue;
        }

        // Use ahead position for random inventory drop.
        if rng.f32() < settings.probabilities.random_drop {
            let target_element_entity = grid_elements.entity(ahead_position);
            commands.drop(ant_entity, ahead_position, *target_element_entity, AtNest);
            continue;
        }

        // Check if hitting a solid element and, if so, consider digging through it.
        let entity = grid_elements.entity(ahead_position);
        let element = grid_elements.element(*entity);
        if *element != Element::Air {
            continue;
        }

        // Avoid dropping inventory when facing upwards since it'll fall on the ant.
        if orientation.is_facing_north() {
            continue;
        }

        // There is an air gap directly ahead of the ant. Consider dropping inventory.
        let inventory_item_element = elements_query.get(inventory.0.unwrap()).unwrap();

        // Prioritize dropping sand above ground and food below ground.
        let drop_sand = *inventory_item_element == Element::Sand
            && nest.is_aboveground(&ahead_position)
            && rng.f32() < settings.probabilities.above_surface_sand_drop;

        let mut drop_food = false;
        if *inventory_item_element == Element::Food {
            if nest.is_underground(&ahead_position) {
                // Don't let ants drop food in tunnels that don't have space for them to navigate around dropped food.
                if grid_elements.is(
                    orientation.get_above_position(&ahead_position),
                    Element::Air,
                ) && grid_elements.is(orientation.get_above_position(position), Element::Air)
                {
                    drop_food = rng.f32() < settings.probabilities.below_surface_food_drop;

                    // If ant is adjacent to food then strongly consider dropping food (creates food piles)
                    let is_food_below =
                        grid_elements.is(orientation.get_below_position(position), Element::Food);

                    if is_food_below
                        && rng.f32() < settings.probabilities.below_surface_food_adjacent_food_drop
                    {
                        drop_food = true;
                    }
                }
            } else {
                if *role == AntRole::Queen {
                    drop_food = rng.f32() < settings.probabilities.above_surface_queen_food_drop;
                }
            }
        }

        if drop_sand || drop_food {
            // Drop inventory in front of ant
            let target_element_entity = grid_elements.entity(ahead_position);
            commands.drop(ant_entity, ahead_position, *target_element_entity, AtNest);
            continue;
        }
    }
}
