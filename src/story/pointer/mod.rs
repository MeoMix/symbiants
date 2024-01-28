use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::story::ui::story::action_menu::PointerAction;

use super::rendering::{
    camera::RenderingCamera,
    common::{SelectedEntity, VisibleGrid},
};

use simulation::{
    common::{grid::Grid, position::Position},
    external_event::ExternalSimulationEvent,
    nest_simulation::{ant::Ant, nest::Nest},
};

pub fn pointer_action_to_simulation_event(
    pointer_action: PointerAction,
    position: Position,
) -> ExternalSimulationEvent {
    match pointer_action {
        PointerAction::Select => {
            panic!("Cannot convert PointerAction::Select to ExternalSimulationEvent")
        }
        PointerAction::DespawnElement => ExternalSimulationEvent::DespawnElement(position),
        PointerAction::SpawnFood => ExternalSimulationEvent::SpawnFood(position),
        PointerAction::SpawnDirt => ExternalSimulationEvent::SpawnDirt(position),
        PointerAction::SpawnSand => ExternalSimulationEvent::SpawnSand(position),
        PointerAction::KillAnt => ExternalSimulationEvent::KillAnt(position),
        PointerAction::SpawnWorkerAnt => ExternalSimulationEvent::SpawnWorkerAnt(position),
        PointerAction::DespawnWorkerAnt => ExternalSimulationEvent::DespawnWorkerAnt(position),
    }
}

#[derive(Resource, Default)]
pub struct PointerTapState {
    pub position: Option<Vec2>,
}

pub fn initialize_pointer_resources(mut commands: Commands) {
    commands.init_resource::<PointerTapState>();
}

pub fn remove_pointer_resources(mut commands: Commands) {
    commands.remove_resource::<PointerTapState>();
}

const DRAG_THRESHOLD: f32 = 4.0;

// Map user input to simulation events which will be processed manually at the start of the next simulation run.
// This needs to occur because events aren't reliably read from within systems which don't necessarily run this/next frame.
pub fn handle_pointer_tap(
    mouse_input: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<RenderingCamera>>,
    grid_query: Query<&Grid>,
    visible_grid: Res<VisibleGrid>,
    is_pointer_captured: Res<IsPointerCaptured>,
    pointer_action: Res<PointerAction>,
    mut external_simulation_event_writer: EventWriter<ExternalSimulationEvent>,
    mut pointer_tap_state: ResMut<PointerTapState>,
    ants_query: Query<(Entity, &Position), With<Ant>>,
    nest_query: Query<&Grid, With<Nest>>,
    mut selected_entity: ResMut<SelectedEntity>,
) {
    if is_pointer_captured.0 {
        return;
    }

    let window = match primary_window_query.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    let pointer_pressed_position = get_pointer_pressed_position(window, &mouse_input, &touches);
    if pointer_pressed_position.is_some() {
        pointer_tap_state.position = pointer_pressed_position;
    }

    if pointer_tap_state.position.is_none() {
        return;
    }

    let pointer_released_position = get_pointer_released_position(window, &mouse_input, &touches);
    if pointer_released_position.is_none() {
        return;
    }

    let pointer_distance = pointer_tap_state
        .position
        .unwrap()
        .distance(pointer_released_position.unwrap());
    if pointer_distance >= DRAG_THRESHOLD {
        return;
    }

    let (camera, camera_transform) = camera_query.single_mut();
    let world_position = camera
        .viewport_to_world_2d(camera_transform, pointer_tap_state.position.unwrap())
        .unwrap();

    let grid_position = grid_query
        .get(visible_grid.0.unwrap())
        .unwrap()
        .world_to_grid_position(world_position);

    if *pointer_action != PointerAction::Select {
        external_simulation_event_writer.send(pointer_action_to_simulation_event(
            *pointer_action,
            grid_position,
        ));

        return;
    }

    let nest = nest_query.single();

    // TODO: Support multiple ants at a given position. Need to select them in a fixed order so that there's a "last ant" so that selecting Element is possible afterward.
    let ant_entity_at_position = ants_query
        .iter()
        .find(|(_, &position)| position == grid_position)
        .map(|(entity, _)| entity);

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

fn get_pointer_pressed_position(
    window: &Window,
    mouse_input: &Res<Input<MouseButton>>,
    touches: &Res<Touches>,
) -> Option<Vec2> {
    if mouse_input.just_pressed(MouseButton::Left) {
        window.cursor_position()
    } else if touches.any_just_pressed() && touches.iter().count() == 1 {
        Some(touches.iter().next()?.position())
    } else {
        None
    }
}

fn get_pointer_released_position(
    window: &Window,
    mouse_input: &Res<Input<MouseButton>>,
    touches: &Res<Touches>,
) -> Option<Vec2> {
    if mouse_input.just_released(MouseButton::Left) {
        window.cursor_position()
    } else if touches.any_just_released() && touches.iter().count() == 1 {
        Some(touches.iter().next()?.position())
    } else {
        None
    }
}

#[derive(Resource, Default, PartialEq)]
pub struct IsPointerCaptured(pub bool);

#[derive(Component)]
pub struct NoPointerCapture;

pub fn is_pointer_captured(
    mut is_pointer_captured: ResMut<IsPointerCaptured>,
    mut contexts: EguiContexts,
) {
    let context = contexts.ctx_mut();
    is_pointer_captured.0 = context.wants_pointer_input() || context.wants_keyboard_input();
}
