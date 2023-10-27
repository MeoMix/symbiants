use crate::ant::commands::AntCommandsExt;
use crate::common::IdMap;
use crate::pointer::ExternalSimulationEvent;
use bevy::prelude::*;
use bevy_turborand::GlobalRng;

use crate::ui::selection_menu::Selected;
use crate::{
    ant::{
        Angle, Ant, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead, Facing,
        Initiative,
    },
    element::{commands::ElementCommandsExt, Element},
    name_list::get_random_name,
    settings::Settings,
    ui::action_menu::PointerAction,
    world_map::{position::Position, WorldMap},
};

/// Process user input events at the start of the FixedUpdate simulation loop.
/// Need to process them manually because they'd be cleared at the end of the next Update
/// which might occur before the next time FixedUpdate runs.
pub fn process_external_event(
    mut external_simulation_events: ResMut<Events<ExternalSimulationEvent>>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    elements_query: Query<&Element>,
    ants_query: Query<
        (
            Entity,
            &Position,
            &AntRole,
            &AntInventory,
            Option<&Initiative>,
        ),
        With<Ant>,
    >,
    selected_entity_query: Query<Entity, With<Selected>>,
    id_map: Res<IdMap>,
) {
    for event in external_simulation_events.drain() {
        let pointer_action = event.action;
        let grid_position = event.position;

        if pointer_action == PointerAction::Select {
            // TODO: Support multiple ants at a given position. Need to select them in a fixed order so that there's a "last ant" so that selecting Element is possible afterward.
            let ant_entity_at_position = ants_query
                .iter()
                .find(|(_, &position, _, _, _)| position == grid_position)
                .map(|(entity, _, _, _, _)| entity);

            let element_entity_at_position = world_map.get_element_entity(grid_position);

            let currently_selected_entity = selected_entity_query.get_single();

            if let Ok(currently_selected_entity) = currently_selected_entity {
                commands
                    .entity(currently_selected_entity)
                    .remove::<Selected>();
            }

            if let Some(ant_entity) = ant_entity_at_position {
                // If tapping on an already selected ant then consider selecting element underneath ant instead.
                if ant_entity_at_position == currently_selected_entity.ok() {
                    if let Some(element_entity) = element_entity_at_position {
                        commands.entity(*element_entity).insert(Selected);
                    } else {
                        commands.entity(ant_entity).remove::<Selected>();
                    }
                } else {
                    commands.entity(ant_entity).insert(Selected);
                }
            } else if let Some(element_entity) = element_entity_at_position {
                if element_entity_at_position == currently_selected_entity.ok().as_ref() {
                    commands.entity(*element_entity).remove::<Selected>();
                } else {
                    commands.entity(*element_entity).insert(Selected);
                }
            }
        } else if pointer_action == PointerAction::Food {
            if world_map.is_element(&elements_query, grid_position, Element::Air) {
                if let Some(entity) = world_map.get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Food, *entity);
                }
            }
        } else if pointer_action == PointerAction::Sand {
            if world_map.is_element(&elements_query, grid_position, Element::Air) {
                if let Some(entity) = world_map.get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Sand, *entity);
                }
            }
        } else if pointer_action == PointerAction::Dirt {
            if world_map.is_element(&elements_query, grid_position, Element::Air) {
                if let Some(entity) = world_map.get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Dirt, *entity);
                }
            }
        } else if pointer_action == PointerAction::DespawnElement {
            if let Some(entity) = world_map.get_element_entity(grid_position) {
                commands.replace_element(grid_position, Element::Air, *entity);
            }
        } else if pointer_action == PointerAction::SpawnWorkerAnt {
            if world_map.is_element(&elements_query, grid_position, Element::Air) {
                commands.spawn_ant(
                    grid_position,
                    AntColor(settings.ant_color),
                    AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                    AntInventory::default(),
                    AntRole::Worker,
                    AntName(get_random_name(&mut rng.reborrow())),
                    Initiative::new(&mut rng.reborrow()),
                );
            }
        } else if pointer_action == PointerAction::KillAnt {
            if let Some((entity, _, _, _, _)) = ants_query
                .iter()
                .find(|(_, &position, _, _, _)| position == grid_position)
            {
                commands.entity(entity).insert(Dead).remove::<Initiative>();
            }
        } else if pointer_action == PointerAction::DespawnWorkerAnt {
            if let Some((ant_entity, _, _, inventory, initiative)) =
                ants_query.iter().find(|(_, &position, &role, _, _)| {
                    position == grid_position && role == AntRole::Worker
                })
            {
                // If the ant is carrying something - drop it first.
                if inventory.0 != None {
                    // If the ant is standing on air then drop element where standing otherwise despawn element.
                    // TODO: in the future maybe try to find an adjacent place to drop element.
                    let element_entity = world_map.get_element_entity(grid_position).unwrap();

                    // TODO: Feels weird to need to care about initative when the user is forcing actions to occur.
                    // Need to consider initative because ants don't regain the ability to act every tick - it takes ~3-5.
                    let can_act = initiative.is_some() && initiative.unwrap().can_act();

                    if world_map.is_element(&elements_query, grid_position, Element::Air) && can_act
                    {
                        commands.drop(ant_entity, grid_position, *element_entity);
                    } else {
                        // No room - despawn inventory.
                        if let Some(inventory_element_id) = &inventory.0 {
                            let element_entity = id_map.0.get(inventory_element_id).unwrap();
                            commands.entity(*element_entity).despawn();
                        }
                    }
                }

                // TODO: I wanted to give ant an "emote" child, need to keep this in mind here since despawn recursive affects it, seems OK tho.
                // despawn_recursive to clean up any existing inventory UI since ant inventory system won't work since ant is gone.
                commands.entity(ant_entity).despawn_recursive();
            }
        } else {
            info!("Not yet supported");
        }
    }
}
