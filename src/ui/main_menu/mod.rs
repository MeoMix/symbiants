use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story_state::StoryState;

// TODO: Sandbox and About menu buttons
pub fn update_main_menu_dialog(
    mut contexts: EguiContexts,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Choose Story Mode")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Welcome to Symbiants");

            ui.vertical_centered(|ui| {
                if ui.button("Begin Story").clicked() {
                    story_state.set(StoryState::Creating);
                }
            });
        });
}
