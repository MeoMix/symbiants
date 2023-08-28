use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    element::{commands::ElementCommandsExt, Element},
    food::FoodCount,
    grid::{position::Position, WorldMap},
};

pub fn handle_mouse_clicks(
    mouse_input: Res<Input<MouseButton>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    mut food_count: ResMut<FoodCount>,
    is_pointer_captured: Res<IsPointerCaptured>,
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

    if mouse_input.just_pressed(MouseButton::Left) {
        handle_left_click(&elements_query, &world_map, &mut commands, &mut food_count, grid_position);
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        handle_right_click(&world_map, &mut commands, grid_position);
    }
}

fn world_to_grid_position(world_map: &WorldMap, world_position: Vec2) -> Position {
    let x = world_position.x + (*world_map.width() as f32 / 2.0) - 0.5;
    let y = -world_position.y + (*world_map.height() as f32 / 2.0) - 0.5;

    Position {
        x: x.abs().round() as isize,
        y: y.abs().round() as isize
    }
}

fn handle_left_click(
    elements_query: &Query<&Element>,
    world_map: &Res<WorldMap>,
    commands: &mut Commands,
    food_count: &mut ResMut<FoodCount>,
    grid_position: Position,
) {
    if world_map.is_element(elements_query, grid_position, Element::Air) && food_count.0 > 0 {
        if let Some(entity) = world_map.get_element(grid_position) {
            commands.replace_element(grid_position, Element::Food, *entity);
            food_count.0 -= 1;
        }
    }
}

fn handle_right_click(
    world_map: &Res<WorldMap>,
    commands: &mut Commands,
    grid_position: Position,
) {
    if let Some(entity) = world_map.get_element(grid_position) {
        commands.replace_element(grid_position, Element::Air, *entity);
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
