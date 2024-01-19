pub mod external_event;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::story::{
    camera::MainCamera, common::position::Position, grid::Grid, ui::action_menu::PointerAction,
};

use super::nest_rendering::common::VisibleGrid;

#[derive(Event, PartialEq, Copy, Clone, Debug)]
pub enum ExternalSimulationEvent {
    Select(Position),
    DespawnElement(Position),
    SpawnFood(Position),
    SpawnDirt(Position),
    SpawnSand(Position),
    KillAnt(Position),
    SpawnWorkerAnt(Position),
    DespawnWorkerAnt(Position),
    ShowCrater,
    ShowNest,
}

impl From<(PointerAction, Position)> for ExternalSimulationEvent {
    fn from((pointer_action, position): (PointerAction, Position)) -> Self {
        match pointer_action {
            PointerAction::Select => ExternalSimulationEvent::Select(position),
            PointerAction::DespawnElement => ExternalSimulationEvent::DespawnElement(position),
            PointerAction::SpawnFood => ExternalSimulationEvent::SpawnFood(position),
            PointerAction::SpawnDirt => ExternalSimulationEvent::SpawnDirt(position),
            PointerAction::SpawnSand => ExternalSimulationEvent::SpawnSand(position),
            PointerAction::KillAnt => ExternalSimulationEvent::KillAnt(position),
            PointerAction::SpawnWorkerAnt => ExternalSimulationEvent::SpawnWorkerAnt(position),
            PointerAction::DespawnWorkerAnt => ExternalSimulationEvent::DespawnWorkerAnt(position),
        }
    }
}

#[derive(Resource, Default)]
pub struct PointerTapState {
    pub position: Option<Vec2>,
}

pub fn initialize_pointer_resources(mut commands: Commands) {
    // Calling init_resource prevents Bevy's automatic event cleanup. Need to do it manually.
    commands.init_resource::<Events<ExternalSimulationEvent>>();
    commands.init_resource::<PointerTapState>();
}

pub fn remove_pointer_resources(mut commands: Commands) {
    commands.remove_resource::<Events<ExternalSimulationEvent>>();
    commands.remove_resource::<PointerTapState>();
}

const DRAG_THRESHOLD: f32 = 4.0;

// Map user input to simulation events which will be processed manually at the start of the next simulation run.
// This needs to occur because events aren't reliably read from within systems which don't necessarily run this/next frame.
pub fn handle_pointer_tap(
    mouse_input: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    grid_query: Query<&Grid>,
    visible_grid: Res<VisibleGrid>,
    is_pointer_captured: Res<IsPointerCaptured>,
    pointer_action: Res<PointerAction>,
    mut external_simulation_event_writer: EventWriter<ExternalSimulationEvent>,
    mut pointer_tap_state: ResMut<PointerTapState>,
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

    external_simulation_event_writer.send(ExternalSimulationEvent::from((
        *pointer_action,
        grid_position,
    )));
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

    // NOTE: 99% of the time just checking wanting_input is fine, but if you move really quickly then there's a brief moment
    // where wanting input isn't true. This can cause the underlying window to get panned undesirably. So, check over area, too.
    let is_pointer_over_egui = context.is_pointer_over_area();
    let is_egui_wanting_input = context.wants_pointer_input() || context.wants_keyboard_input();

    is_pointer_captured.0 = is_egui_wanting_input || is_pointer_over_egui;
}
