use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story_state::StoryState;

pub fn update_story_over_dialog(
    mut contexts: EguiContexts,
    mut story_state: ResMut<NextState<StoryState>>,
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
                    story_state.set(StoryState::Cleanup);
                }
            });
        });
}
