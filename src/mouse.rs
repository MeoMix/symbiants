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
    let Ok(window) = primary_window_query.get_single() else {
        return;
    };

    if mouse_input.just_pressed(MouseButton::Left) {
        if is_pointer_captured.0 {
            return;
        }

        let (camera, camera_transform) = query.single_mut();

        let cursor_position = match window.cursor_position() {
            Some(position) => position,
            None => return,
        };

        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();

        // Convert from world to local position
        let x = world_position.x + (*world_map.width() as f32 / 2.0) - 0.5;
        let y = -world_position.y + (*world_map.height() as f32 / 2.0) - 0.5;

        // TODO: is abs + round the right way to do this? maybe round() will off-by-one?
        let grid_position = Position {
            x: x.abs().round() as isize,
            y: y.abs().round() as isize
        };

        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            if food_count.0 > 0 {
                let Some(entity) = world_map.get_element(grid_position) else {
                    return;
                };
                info!("replace_element: {:?}", grid_position);
                commands.replace_element(grid_position, Element::Food, *entity);

                food_count.0 -= 1;
            }
        }
    }

    if mouse_input.just_pressed(MouseButton::Right) {
        if is_pointer_captured.0 {
            return;
        }

        let (camera, camera_transform) = query.single_mut();

        let cursor_position = match window.cursor_position() {
            Some(position) => position,
            None => return,
        };

        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();

        let grid_position = Position {
            x: world_position.x.abs().floor() as isize,
            y: world_position.y.abs().floor() as isize,
        };

        let Some(entity) = world_map.get_element(grid_position) else {
            return;
        };
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
