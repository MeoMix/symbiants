use crate::ant::commands::AntCommandsExt;
use crate::common::IdMap;
use bevy::input::touch::Touch;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use bevy_turborand::GlobalRng;

use crate::ui::selection_menu::Selected;
use crate::{
    ant::{
        Angle, Ant, AntColor, AntInventory, AntLabel, AntName, AntOrientation, AntRole, Dead,
        Facing, Initiative,
    },
    camera::MainCamera,
    element::{commands::ElementCommandsExt, Element},
    name_list::get_random_name,
    settings::Settings,
    ui::action_menu::PointerAction,
    world_map::{position::Position, WorldMap},
};

pub fn handle_pointer_tap(
    mouse_input: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    elements_query: Query<&Element>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    is_pointer_captured: Res<IsPointerCaptured>,
    pointer_action: Res<PointerAction>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    ants_query: Query<(Entity, &Position, &AntRole, &AntInventory, Option<&Initiative>), With<Ant>>,
    ant_label_query: Query<(Entity, &AntLabel)>,
    selected_entity_query: Query<Entity, With<Selected>>,
    id_map: Res<IdMap>,
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

    if *pointer_action == PointerAction::Select {
        // TODO: Support multiple ants at a given position. Need to select them in a fixed order so that there's a "last ant" so that selecting Element is possible afterward.
        let selected_ant_entity = ants_query
            .iter()
            .find(|(_, &position, _, _, _)| position == grid_position)
            .map(|(entity, _, _, _, _)| entity);

        let selected_element_entity = world_map.get_element_entity(grid_position);

        let currently_selected_entity = selected_entity_query.get_single();

        if let Ok(currently_selected_entity) = currently_selected_entity {
            commands
                .entity(currently_selected_entity)
                .remove::<Selected>();
        }

        if let Some(ant_entity) = selected_ant_entity {
            // If tapping on an already selected ant then consider selecting element underneath ant instead.
            if selected_ant_entity == currently_selected_entity.ok() {
                if let Some(element_entity) = selected_element_entity {
                    commands.entity(*element_entity).insert(Selected);
                }
            } else {
                commands.entity(ant_entity).insert(Selected);
            }
        } else if let Some(element_entity) = selected_element_entity {
            commands.entity(*element_entity).insert(Selected);
        }
    } else if *pointer_action == PointerAction::Food {
        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            if let Some(entity) = world_map.get_element_entity(grid_position) {
                commands.replace_element(grid_position, Element::Food, *entity);
            }
        }
    } else if *pointer_action == PointerAction::Sand {
        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            if let Some(entity) = world_map.get_element_entity(grid_position) {
                commands.replace_element(grid_position, Element::Sand, *entity);
            }
        }
    } else if *pointer_action == PointerAction::Dirt {
        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            if let Some(entity) = world_map.get_element_entity(grid_position) {
                commands.replace_element(grid_position, Element::Dirt, *entity);
            }
        }
    } else if *pointer_action == PointerAction::DespawnElement {
        if let Some(entity) = world_map.get_element_entity(grid_position) {
            commands.replace_element(grid_position, Element::Air, *entity);
        }
    } else if *pointer_action == PointerAction::SpawnWorkerAnt {
        if world_map.is_element(&elements_query, grid_position, Element::Air) {
            commands.spawn_ant(
                grid_position,
                AntColor(settings.ant_color),
                AntOrientation::new(Facing::random(&mut rng.reborrow()), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName(get_random_name(&mut rng.reborrow())),
                Initiative::new(&mut rng.reborrow()),
            );
        }
    } else if *pointer_action == PointerAction::KillAnt {
        if let Some((entity, _, _, _, _)) = ants_query
            .iter()
            .find(|(_, &position, _, _, _)| position == grid_position)
        {
            commands.entity(entity).insert(Dead).remove::<Initiative>();
        }
    } else if *pointer_action == PointerAction::DespawnWorkerAnt {
        if let Some((ant_entity, _, _, inventory, initiative)) = ants_query
            .iter()
            .find(|(_, &position, &role, _, _)| position == grid_position && role == AntRole::Worker)
        {
            // If the ant is carrying something - drop it first.
            if inventory.0 != None {
                // If the ant is standing on air then drop element where standing otherwise despawn element.
                // TODO: in the future maybe try to find an adjacent place to drop element.
                let element_entity = world_map.get_element_entity(grid_position).unwrap();

                // TODO: Feels weird to need to care about initative when the user is forcing actions to occur.
                let can_act = initiative.is_some() && initiative.unwrap().can_act();

                if world_map.is_element(&elements_query, grid_position, Element::Air) && can_act {
                    commands.drop(ant_entity, grid_position, *element_entity);
                } else {
                    // No room - despawn inventory.
                    if let Some(inventory_element_id) = &inventory.0 {
                        let element_entity = id_map.0.get(inventory_element_id).unwrap();
                        commands.entity(*element_entity).despawn();
                    }
                }
            }

            // despawn_recursive to clean up any existing inventory UI since ant inventory system won't work since ant is gone.
            commands.entity(ant_entity).despawn_recursive();

            // TODO: I tried using RemovedComponents in an attempt to react to any time an ant is despawned but it didn't seem to work

            // Unfortunately, need to despawn label separately.
            let (label_entity, _) = ant_label_query
                .iter()
                .find(|(_, label)| label.0 == ant_entity)
                .unwrap();

            commands.entity(label_entity).despawn();
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
