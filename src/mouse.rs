use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    elements::{is_element, Element, FoodElementBundle},
    map::{Position, WorldMap},
};

pub fn handle_mouse_clicks(
    mouse_input: Res<Input<MouseButton>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    mut world_map: ResMut<WorldMap>,
) {
    let Ok(window) = primary_window_query.get_single() else { return };

    if mouse_input.just_released(MouseButton::Left) {
        let (camera, camera_transform) = query.single_mut();

        let cursor_position = window.cursor_position().unwrap();
        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor_position)
            .unwrap();

        let grid_position = Position {
            x: world_position.x.abs().floor() as isize,
            y: world_position.y.abs().floor() as isize,
        };

        if is_element(&world_map, &elements_query, &grid_position, &Element::Air) {
            let food_entity = commands.spawn(FoodElementBundle::new(grid_position)).id();
            world_map.set_element(grid_position, food_entity);
        }
    }
}
