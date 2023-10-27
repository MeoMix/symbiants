use bevy::input::touch::Touch;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;

use crate::{
    camera::MainCamera,
    ui::action_menu::PointerAction,
    world_map::{position::Position, WorldMap},
};

#[derive(Event)]
pub struct ExternalSimulationEvent {
    // TODO: naming of PointerAction breaks encapsulation
    pub action: PointerAction,
    pub position: Position,
}

pub fn setup_pointer(mut commands: Commands) {
    // Calling init_resource prevents Bevy's automatic event cleanup. Need to do it manually.
    commands.init_resource::<Events<ExternalSimulationEvent>>();
}

// Map user input to simulation events which will be processed manually at the start of the next simulation run.
// This needs to occur because events aren't reliably read from within systems which don't necessarily run this/next frame.
pub fn handle_pointer_tap(
    mouse_input: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    world_map: Res<WorldMap>,
    is_pointer_captured: Res<IsPointerCaptured>,
    pointer_action: Res<PointerAction>,
    mut external_simulation_event_writer: EventWriter<ExternalSimulationEvent>,
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

    let pointer_position;
    if left_mouse_button_pressed {
        pointer_position = match window.cursor_position() {
            Some(position) => position,
            None => return,
        };
    } else if primary_touch_pressed {
        pointer_position = touches_vec[0].position();
    } else {
        return;
    }

    let (camera, camera_transform) = camera_query.single_mut();

    let world_position = camera
        .viewport_to_world_2d(camera_transform, pointer_position)
        .unwrap();

    let grid_position = world_to_grid_position(&world_map, world_position);

    external_simulation_event_writer.send(ExternalSimulationEvent {
        action: *pointer_action,
        position: grid_position,
    });
}

fn world_to_grid_position(world_map: &WorldMap, world_position: Vec2) -> Position {
    let x = world_position.x + (*world_map.width() as f32 / 2.0) - 0.5;
    let y = -world_position.y + (*world_map.height() as f32 / 2.0) - 0.5;

    Position {
        x: x.abs().round() as isize,
        y: y.abs().round() as isize,
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
