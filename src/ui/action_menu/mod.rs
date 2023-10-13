// Create a floating menu which contains a set of action icons. Very similar to Photoshop/Paint action menu.
// Used in Sandbox Mode to allow the user to play around with the environment - manually spawning/despawning anything that could exist.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

#[derive(Resource, Default, PartialEq, Copy, Clone, Debug)]
pub enum PointerAction {
    #[default]
    Select,
    DespawnElement,
    Food,
    Dirt,
    Sand,
    KillAnt,
    SpawnWorkerAnt,
    DespawnWorkerAnt,
}

pub fn setup_action_menu(mut commands: Commands) {
    commands.init_resource::<PointerAction>();
}

pub fn teardown_action_menu(mut commands: Commands) {
    commands.remove_resource::<PointerAction>();
}

pub fn update_action_menu(
    mut contexts: EguiContexts,
    mut pointer_action: ResMut<PointerAction>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    // TODO: resetting story doesn't reset window position
    egui::Window::new("Actions")
        .default_pos(egui::Pos2::new(window.width(), 0.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Select, "Select");
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::DespawnElement,
                "Remove Element",
            );
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Food, "Place Food");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Dirt, "Place Dirt");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Sand, "Place Sand");

            ui.selectable_value(pointer_action.as_mut(), PointerAction::KillAnt, "Kill Ant");
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::SpawnWorkerAnt,
                "Place Worker Ant",
            );
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::DespawnWorkerAnt,
                "Remove Worker Ant",
            );
        });
}
