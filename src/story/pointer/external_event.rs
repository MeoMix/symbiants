use crate::story::{
    ant::commands::AntCommandsExt,
    common::position::Position,
    grid::{Grid, VisibleGridState},
    rendering::common::{SelectedEntity, VisibleGrid},
    simulation::{
        crater_simulation::crater::Crater,
        nest_simulation::nest::{AtNest, Nest},
    },
};

use bevy::prelude::*;
use bevy_turborand::GlobalRng;

use crate::story::{
    ant::{
        Angle, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead, Facing, Initiative,
    },
    element::{commands::ElementCommandsExt, Element},
};

use crate::settings::Settings;

use super::ExternalSimulationEvent;

/// Process user input events at the start of the FixedUpdate simulation loop.
/// Need to process them manually because they'd be cleared at the end of the next Update
/// which might occur before the next time FixedUpdate runs.
pub fn process_external_event(
    mut external_simulation_events: ResMut<Events<ExternalSimulationEvent>>,
    mut commands: Commands,
    nest_query: Query<(Entity, &Grid), With<Nest>>,
    crater_query: Query<(Entity, &Grid), With<Crater>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    elements_query: Query<&Element>,
    ants_query: Query<(Entity, &Position, &AntRole, &AntInventory)>,
    mut next_visible_grid_state: ResMut<NextState<VisibleGridState>>,
    mut selected_entity: ResMut<SelectedEntity>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    let (_, nest) = nest_query.single();

    for event in external_simulation_events.drain() {
        match event {
            ExternalSimulationEvent::ShowCrater => {
                visible_grid.0 = None; // Some(crater_query.single().0);
                next_visible_grid_state.set(VisibleGridState::Crater);
            }
            ExternalSimulationEvent::ShowNest => {
                visible_grid.0 = Some(nest_query.single().0);
                next_visible_grid_state.set(VisibleGridState::Nest);
            }
            ExternalSimulationEvent::Select(grid_position) => {
                // TODO: Support multiple ants at a given position. Need to select them in a fixed order so that there's a "last ant" so that selecting Element is possible afterward.
                let ant_entity_at_position = ants_query
                    .iter()
                    .find(|(_, &position, _, _)| position == grid_position)
                    .map(|(entity, _, _, _)| entity);

                let element_entity_at_position = nest.elements().get_element_entity(grid_position);

                let currently_selected_entity = selected_entity.0;

                if let Some(ant_entity) = ant_entity_at_position {
                    // If tapping on an already selected ant then consider selecting element underneath ant instead.
                    if ant_entity_at_position == currently_selected_entity {
                        if let Some(element_entity) = element_entity_at_position {
                            selected_entity.0 = Some(*element_entity);
                        } else {
                            selected_entity.0 = None;
                        }
                    } else {
                        // If there is an ant at the given position, and it's not selected, but the element underneath it is selected
                        // then assume user wants to deselect element and not select the ant. They can select again after if they want the ant.
                        if element_entity_at_position == currently_selected_entity.as_ref() {
                            selected_entity.0 = None;
                        } else {
                            selected_entity.0 = Some(ant_entity);
                        }
                    }
                } else if let Some(element_entity) = element_entity_at_position {
                    if element_entity_at_position == currently_selected_entity.as_ref() {
                        selected_entity.0 = None;
                    } else {
                        selected_entity.0 = Some(*element_entity);
                    }
                } else {
                    selected_entity.0 = None;
                }
            }
            ExternalSimulationEvent::SpawnFood(grid_position) => {
                if nest
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = nest.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Food, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnSand(grid_position) => {
                if nest
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = nest.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Sand, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnDirt(grid_position) => {
                if nest
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = nest.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Dirt, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::DespawnElement(grid_position) => {
                if let Some(entity) = nest.elements().get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Air, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnWorkerAnt(grid_position) => {
                if nest
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    commands.spawn_ant(
                        grid_position,
                        AntColor(settings.ant_color),
                        AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                        AntInventory::default(),
                        AntRole::Worker,
                        AntName::random(&mut rng.reborrow()),
                        Initiative::new(&mut rng.reborrow()),
                        AtNest,
                    );
                }
            }
            ExternalSimulationEvent::KillAnt(grid_position) => {
                if let Some((entity, _, _, _)) = ants_query
                    .iter()
                    .find(|(_, &position, _, _)| position == grid_position)
                {
                    commands.entity(entity).insert(Dead).remove::<Initiative>();
                }
            }
            ExternalSimulationEvent::DespawnWorkerAnt(grid_position) => {
                if let Some((ant_entity, _, _, inventory)) =
                    ants_query.iter().find(|(_, &position, &role, _)| {
                        position == grid_position && role == AntRole::Worker
                    })
                {
                    // TODO: This should happen automatically when an ant is despawned
                    if let Some(element_entity) = &inventory.0 {
                        commands.entity(*element_entity).despawn();
                    }

                    commands.entity(ant_entity).despawn_recursive();
                }
            }
        }
    }
}
