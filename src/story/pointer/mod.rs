pub mod external_event;

use bevy::input::touch::Touch;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::story::{
    camera::MainCamera,
    common::position::Position,
    grid::{Grid, VisibleGrid},
    ui::action_menu::PointerAction,
};

#[derive(Event)]
pub struct ExternalSimulationEvent {
    // TODO: naming of PointerAction breaks encapsulation
    pub action: PointerAction,
    pub position: Option<Position>,
}

#[derive(Resource, Default)]
pub struct PointerTapState {
    pub position: Option<Vec2>,
}

pub fn setup_pointer(mut commands: Commands) {
    // Calling init_resource prevents Bevy's automatic event cleanup. Need to do it manually.
    commands.init_resource::<Events<ExternalSimulationEvent>>();
    commands.init_resource::<PointerTapState>();
}

// Map user input to simulation events which will be processed manually at the start of the next simulation run.
// This needs to occur because events aren't reliably read from within systems which don't necessarily run this/next frame.
pub fn handle_pointer_tap(
    mouse_input: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    visible_grid_query: Query<&Grid, With<VisibleGrid>>,
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

    let left_mouse_button_pressed = mouse_input.just_pressed(MouseButton::Left);
    let touches_vec: Vec<&Touch> = touches.iter().collect();
    let primary_touch_pressed = touches.any_just_pressed() && touches_vec.len() == 1;

    let pointer_pressed_position = if left_mouse_button_pressed {
        match window.cursor_position() {
            Some(position) => Some(position),
            None => return,
        }
    } else if primary_touch_pressed {
        Some(touches_vec[0].position())
    } else {
        None
    };

    if pointer_pressed_position.is_some() {
        pointer_tap_state.position = pointer_pressed_position;
    }

    if pointer_tap_state.position.is_none() {
        return;
    }

    let left_mouse_button_released = mouse_input.just_released(MouseButton::Left);
    let primary_touch_released = touches.any_just_released() && touches_vec.len() == 1;

    let pointer_released_position = if left_mouse_button_released {
        match window.cursor_position() {
            Some(position) => Some(position),
            None => return,
        }
    } else if primary_touch_released {
        Some(touches_vec[0].position())
    } else {
        None
    };

    if pointer_released_position.is_none() {
        return;
    }

    let pointer_distance = pointer_tap_state
        .position
        .unwrap()
        .distance(pointer_released_position.unwrap());
    let is_drag = pointer_distance >= 5.0;
    if is_drag {
        return;
    }

    let (camera, camera_transform) = camera_query.single_mut();

    let world_position = camera
        .viewport_to_world_2d(camera_transform, pointer_tap_state.position.unwrap())
        .unwrap();

    let visible_grid = visible_grid_query.single();
    let grid_position = visible_grid.world_to_grid_position(world_position);

    external_simulation_event_writer.send(ExternalSimulationEvent {
        action: *pointer_action,
        position: Some(grid_position),
    });
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
