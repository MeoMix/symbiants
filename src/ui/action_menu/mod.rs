use std::time::Duration;

// Create a floating menu which contains a set of action icons. Very similar to Photoshop/Paint action menu.
// Used in Sandbox Mode to allow the user to play around with the environment - manually spawning/despawning anything that could exist.
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::time::{TicksPerSecond, DEFAULT_TICKS_PER_SECOND, FASTFORWARD_TICKS_PER_SECOND};

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

pub fn initialize_action_menu(mut commands: Commands) {
    commands.init_resource::<PointerAction>();
}

pub fn update_action_menu(
    mut contexts: EguiContexts,
    mut pointer_action: ResMut<PointerAction>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut fixed_time: ResMut<FixedTime>,
    mut ticks_per_second: ResMut<TicksPerSecond>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    // TODO: resetting story doesn't reset window position
    egui::Window::new("Actions")
        .default_pos(egui::Pos2::new(window.width(), 0.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::Select,
                "Select Unit",
            );
            ui.selectable_value(
                pointer_action.as_mut(),
                PointerAction::DespawnElement,
                "Despawn Element",
            );
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Food, "Spawn Food");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Dirt, "Spawn Dirt");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::Sand, "Spawn Sand");
            
            ui.selectable_value(pointer_action.as_mut(), PointerAction::KillAnt, "Kill Ant");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::SpawnWorkerAnt, "Spawn Worker Ant");
            ui.selectable_value(pointer_action.as_mut(), PointerAction::DespawnWorkerAnt, "Despawn Worker Ant");

            if ui.button("Speed++").clicked() {
                let new_ticks_per_second = (ticks_per_second.0 + DEFAULT_TICKS_PER_SECOND)
                    .min(FASTFORWARD_TICKS_PER_SECOND);

                fixed_time.period = Duration::from_secs_f32(1.0 / new_ticks_per_second);
                ticks_per_second.0 = new_ticks_per_second;
            }

            if ui.button("Speed--").clicked() {
                let new_ticks_per_second =
                    (ticks_per_second.0 - DEFAULT_TICKS_PER_SECOND).max(DEFAULT_TICKS_PER_SECOND);

                fixed_time.period = Duration::from_secs_f32(1.0 / new_ticks_per_second);
                ticks_per_second.0 = new_ticks_per_second;
            }
        });
}

// TODO: discrepancy between using world directly vs commands for remove_resource
pub fn deinitialize_action_menu(mut commands: Commands) {
    commands.remove_resource::<PointerAction>();
}
