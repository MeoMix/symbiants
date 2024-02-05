use bevy::prelude::*;
use bevy_turborand::GlobalRng;

use crate::{
    common::{grid::Grid, position::Position, Zone}, crater_simulation::crater::AtCrater, nest_simulation::{
        ant::commands::AntCommandsExt,
        ant::{
            Angle, AntColor, AntInventory, AntName, AntOrientation, AntRole, Dead, Facing,
            Initiative,
        },
        element::{commands::ElementCommandsExt, Element},
        nest::{AtNest, Nest},
    }, settings::Settings
};

#[derive(Event, PartialEq, Copy, Clone, Debug)]
pub enum ExternalSimulationEvent<Z: Zone> {
    DespawnElement(Position, Z),
    SpawnFood(Position, Z),
    SpawnDirt(Position, Z),
    SpawnSand(Position, Z),
    KillAnt(Position, Z),
    SpawnWorkerAnt(Position, Z),
    DespawnWorkerAnt(Position, Z),
}

pub fn initialize_external_event_resources(mut commands: Commands) {
    // Calling init_resource prevents Bevy's automatic event cleanup. Need to do it manually.
    commands.init_resource::<Events<ExternalSimulationEvent<AtNest>>>();
    commands.init_resource::<Events<ExternalSimulationEvent<AtCrater>>>();
}

pub fn remove_external_event_resources(mut commands: Commands) {
    commands.remove_resource::<Events<ExternalSimulationEvent<AtNest>>>();
    commands.remove_resource::<Events<ExternalSimulationEvent<AtCrater>>>();
}

/// Process user input events at the start of the FixedUpdate simulation loop.
/// Need to process them manually because they'd be cleared at the end of the next Update
/// which might occur before the next time FixedUpdate runs.
pub fn process_external_event<Z: Zone + Copy>(
    mut external_simulation_events: ResMut<Events<ExternalSimulationEvent<Z>>>,
    mut commands: Commands,
    grid_query: Query<&Grid, With<Z>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    // TODO: Not filtering on <Z> here
    elements_query: Query<&Element>,
    ants_query: Query<(Entity, &Position, &AntRole, &AntInventory), With<Z>>,
) {
    let grid: &Grid = grid_query.single();

    for event in external_simulation_events.drain() {
        match event {
            ExternalSimulationEvent::SpawnFood(grid_position, zone) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Food, *entity, zone);
                }
            }
            ExternalSimulationEvent::SpawnSand(grid_position, zone) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Sand, *entity, zone);
                }
            }
            ExternalSimulationEvent::SpawnDirt(grid_position, zone) => {
                if grid
                    .elements()
                    .is_element(&elements_query, grid_position, Element::Air)
                {
                    let entity = grid.elements().element_entity(grid_position);
                    commands.replace_element(grid_position, Element::Dirt, *entity, zone);
                }
            }
            ExternalSimulationEvent::DespawnElement(grid_position, zone) => {
                if let Some(entity) = grid.elements().get_element_entity(grid_position) {
                    commands.replace_element(grid_position, Element::Air, *entity, zone);
                }
            }
            ExternalSimulationEvent::SpawnWorkerAnt(grid_position, zone) => {
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
                        zone,
                    );
                }
            }
            ExternalSimulationEvent::KillAnt(grid_position, zone) => {
                if let Some((entity, _, _, _)) = ants_query
                    .iter()
                    .find(|(_, &position, _, _)| position == grid_position)
                {
                    commands.entity(entity).insert(Dead).remove::<Initiative>();
                }
            }
            ExternalSimulationEvent::DespawnWorkerAnt(grid_position, zone) => {
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
