use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::story_state::StoryState;

pub fn update_settings_menu(
    mut contexts: EguiContexts,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    // TODO: resetting story doesn't reset window position
    egui::Window::new("Settings")
        .default_pos(egui::Pos2::new(window.width() - 300.0, 0.0))
        .resizable(false)
        .show(ctx, |ui| {
            if ui.button("Reset Story").clicked() {
                story_state.set(StoryState::Cleanup);
            }

            if ui.button("End Story").clicked() {
                story_state.set(StoryState::Over);
            }
        });
}
