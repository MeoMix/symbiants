use bevy::prelude::*;
use bevy_turborand::GlobalRng;

use crate::{
    common::{grid::Grid, position::Position},
    nest_simulation::{
        ant::commands::AntCommandsExt,
        ant::{
            Angle, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead, Facing,
            Initiative,
        },
        element::{commands::ElementCommandsExt, Element},
        nest::{AtNest, Nest},
    },
    settings::Settings,
};

#[derive(Event, PartialEq, Copy, Clone, Debug)]
pub enum ExternalSimulationEvent {
    DespawnElement(Position),
    SpawnFood(Position),
    SpawnDirt(Position),
    SpawnSand(Position),
    KillAnt(Position),
    SpawnWorkerAnt(Position),
    DespawnWorkerAnt(Position),
}

pub fn initialize_external_event_resources(mut commands: Commands) {
    // Calling init_resource prevents Bevy's automatic event cleanup. Need to do it manually.
    commands.init_resource::<Events<ExternalSimulationEvent>>();
}

pub fn remove_external_event_resources(mut commands: Commands) {
    commands.remove_resource::<Events<ExternalSimulationEvent>>();
}

/// Process user input events at the start of the FixedUpdate simulation loop.
/// Need to process them manually because they'd be cleared at the end of the next Update
/// which might occur before the next time FixedUpdate runs.
pub fn process_external_event(
    mut external_simulation_events: ResMut<Events<ExternalSimulationEvent>>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    elements_query: Query<&Element>,
    ants_query: Query<(Entity, &Position, &AntRole, &AntInventory)>,
) {
    let grid: &Grid = nest_query.single();

    for event in external_simulation_events.drain() {
        match event {
            ExternalSimulationEvent::SpawnFood(grid_position) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Food, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnSand(grid_position) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Sand, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnDirt(grid_position) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Dirt, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::DespawnElement(grid_position) => {
                if let Some(entity) = grid.elements().get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Air, *entity, AtNest);
                }
            }
            ExternalSimulationEvent::SpawnWorkerAnt(grid_position) => {
                if grid
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
