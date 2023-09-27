use crate::{
    ant::birthing::Birthing,
    common::register,
    element::Element,
    grid::{position::Position, WorldMap},
    settings::Settings,
};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

use super::{commands::AntCommandsExt, AntInventory, AntOrientation, Dead, Initiative};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Nesting {
    position: Option<Position>,
}

impl Nesting {
    pub fn is_started(&self) -> bool {
        self.position.is_some()
    }

    pub fn start(&mut self, position: Position) {
        self.position = Some(position);
    }

    pub fn position(&self) -> &Option<Position> {
        &self.position
    }
}

pub fn initialize_nesting(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Nesting>(&app_type_registry, &mut saveable_registry);
}

pub fn ants_nesting(
    mut ants_query: Query<
        (
            &mut Nesting,
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
            &Position,
            Entity,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    for (mut nesting, orientation, inventory, mut initiative, position, ant_entity) in
        ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        if world_map.is_aboveground(&position) && !nesting.is_started() {
            if inventory.0 == None {
                if rng.f32() < settings.probabilities.above_surface_queen_nest_dig {
                    // If x position is within 20% of world edge then don't dig there
                    let offset = settings.world_width / 5;
                    let is_too_near_left_edge = position.x < offset;
                    let is_too_near_right_edge = position.x > settings.world_width - offset;

                    if !is_too_near_left_edge && !is_too_near_right_edge {
                        let target_position =
                            *position + orientation.rotate_forward().get_forward_delta();
                        let target_element_entity = *world_map.element(target_position);
                        commands.dig(ant_entity, target_position, target_element_entity);

                        initiative.consume_action();

                        // TODO: technically this command could fail and I wouldn't want to mark nested?
                        // TODO: replace this with pheromones - queen should be able to find her way back to dig site via pheromones rather than
                        // enforcing nest generation probabilistically
                        nesting.start(target_position);

                        continue;
                    }
                }
            }
        }

        if position.y - world_map.surface_level() > 8 {
            // Check if the queen is sufficiently surounded by space while being deep underground and, if so, decide to start nesting.
            let left_position = *position + Position::NEG_X;
            let above_position = *position + Position::NEG_Y;
            let right_position = *position + Position::X;

            let has_valid_air_nest = world_map.is_all_element(
                &elements_query,
                &[left_position, *position, above_position, right_position],
                Element::Air,
            );

            let below_position = *position + Position::Y;
            // Make sure there's stable place for ant child to be born
            let behind_position = *position + orientation.turn_around().get_forward_delta();
            let behind_below_position = behind_position + Position::Y;

            let has_valid_dirt_nest = world_map.is_all_element(
                &elements_query,
                &[below_position, behind_below_position],
                Element::Dirt,
            );

            if has_valid_air_nest && has_valid_dirt_nest {
                // Stop queen from nesting
                commands.entity(ant_entity).remove::<Nesting>();

                // Spawn birthing component on QueenAnt
                commands.entity(ant_entity).insert(Birthing::default());

                if inventory.0 != None {
                    let target_position = *position + orientation.get_forward_delta();
                    let target_element_entity = world_map.element(target_position);
                    commands.drop(ant_entity, target_position, *target_element_entity);
                }

                initiative.consume_action();
                continue;
            }
        }
    }
}
