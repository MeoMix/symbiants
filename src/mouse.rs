use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    element::{is_element, Element, commands::ElementCommandsExt},
    map::{Position, WorldMap}, food::FoodCount,
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
    let Ok(window) = primary_window_query.get_single() else { return };

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

        let grid_position = Position {
            x: world_position.x.abs().floor() as isize,
            y: world_position.y.abs().floor() as isize,
        };

        if is_element(&world_map, &elements_query, &grid_position, &Element::Air) {
            if food_count.0 > 0 {
                let Some(entity) = world_map.get_element(grid_position) else { return };
                info!("replace_element6: {:?}", grid_position);
                commands.replace_element(grid_position, *entity, Element::Food);

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
        
        let Some(entity) = world_map.get_element(grid_position) else { return };
        commands.replace_element(grid_position, *entity, Element::Air);

        // if is_element(&world_map, &elements_query, &grid_position, &Element::Air) {
        //     if food_count.0 > 0 {
        //         let Some(entity) = world_map.get_element(grid_position) else { return };
        //         info!("replace_element6: {:?}", grid_position);
        //         commands.replace_element(grid_position, *entity, Element::Food);

        //         food_count.0 -= 1;
        //     }
        // }
    }
}


#[derive(Resource)]
pub struct IsPointerCaptured(pub bool);

#[derive(Component)]
pub struct NoPointerCapture;

pub fn is_pointer_captured_system(
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