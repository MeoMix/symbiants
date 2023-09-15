use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    element::{commands::ElementCommandsExt, Element},
    grid::{position::Position, WorldMap},
    ui::action_menu::PointerAction,
};

pub fn handle_mouse_clicks(
    mouse_input: Res<Input<MouseButton>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    is_pointer_captured: Res<IsPointerCaptured>,
    pointer_action: Res<PointerAction>,
) {
    if is_pointer_captured.0 {
        return;
    }

    let window = match primary_window_query.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    let cursor_position = match window.cursor_position() {
        Some(position) => position,
        None => return,
    };

    let (camera, camera_transform) = query.single_mut();

    let world_position = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .unwrap();

    let grid_position = world_to_grid_position(&world_map, world_position);

    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    if *pointer_action == PointerAction::Food {
        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            if let Some(entity) = world_map.get_element(grid_position) {
                commands.replace_element(grid_position, Element::Food, *entity);
            }
        }
    } else if *pointer_action == PointerAction::Despawn {
        if let Some(entity) = world_map.get_element(grid_position) {
            commands.replace_element(grid_position, Element::Air, *entity);
        }
    } else {
        info!("Not yet supported");
    }
}

fn world_to_grid_position(world_map: &WorldMap, world_position: Vec2) -> Position {
    let x = world_position.x + (*world_map.width() as f32 / 2.0) - 0.5;
    let y = -world_position.y + (*world_map.height() as f32 / 2.0) - 0.5;

    Position {
        x: x.abs().round() as isize,
        y: y.abs().round() as isize,
    }
}

#[derive(Resource, Default)]
pub struct IsPointerCaptured(pub bool);

#[derive(Component)]
pub struct NoPointerCapture;

pub fn is_pointer_captured(
    mut is_pointer_captured: ResMut<IsPointerCaptured>,
    interaction_query: Query<
        &Interaction,
        (With<Node>, Changed<Interaction>, Without<NoPointerCapture>),
    >,
) {
    is_pointer_captured.0 = interaction_query
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
}
