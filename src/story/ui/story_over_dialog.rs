use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::app_state::AppState;

pub fn update_story_over_dialog(
    mut contexts: EguiContexts,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Story Over")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label("Queen has died. Sadge :(. Story over. Begin again?");

            ui.vertical_centered(|ui| {
                if ui.button("Begin New Story").clicked() {
                    next_app_state.set(AppState::Cleanup);
                }
            });
        });
}
