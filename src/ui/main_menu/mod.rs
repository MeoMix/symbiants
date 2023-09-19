use crate::story_state::StoryState;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

// TODO: About menu
pub fn update_main_menu_dialog(
    mut contexts: EguiContexts,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Main Menu")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Welcome to Symbiants");

                ui.add_enabled(false, egui::Button::new("Story Mode"))
                    .on_disabled_hover_text("Coming soon™!");

                if ui.button("Sandbox Mode").clicked() {
                    story_state.set(StoryState::Creating);
                }
            });
        });
}
