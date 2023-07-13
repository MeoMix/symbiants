use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::WorldScale,
    elements::{is_element, Element, FoodElementBundle},
    map::{Position, WorldMap},
};

pub fn handle_mouse_clicks(
    mouse_input: Res<Input<MouseButton>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_scale: Res<WorldScale>,
    mut world_map: ResMut<WorldMap>,
) {
    let Ok(window) = primary_window_query.get_single() else { return };

    if mouse_input.just_pressed(MouseButton::Left) {
        info!("click at {:?}", window.cursor_position());

        let cursor_position = window.cursor_position().unwrap();
        let grid_position = Position {
            x: (cursor_position.x / world_scale.0).floor() as isize,
            y: ((window.height() - cursor_position.y) / world_scale.0).floor() as isize,
        };

        info!("grid_position {:?}", grid_position);

        if is_element(&world_map, &elements_query, &grid_position, &Element::Air) {
            let food_entity = commands.spawn(FoodElementBundle::new(grid_position)).id();
            world_map.set_element(grid_position, food_entity);
        }
    }
}
