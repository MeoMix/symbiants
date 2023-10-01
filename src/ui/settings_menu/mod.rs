use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{
    story_state::StoryState,
    time::{TicksPerSecond, DEFAULT_TICKS_PER_SECOND, MAX_USER_TICKS_PER_SECOND},
};

pub fn update_settings_menu(
    mut contexts: EguiContexts,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut story_state: ResMut<NextState<StoryState>>,
    mut ticks_per_second: ResMut<TicksPerSecond>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

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

            ui.add(
                egui::Slider::new(
                    &mut ticks_per_second.0,
                    DEFAULT_TICKS_PER_SECOND..=MAX_USER_TICKS_PER_SECOND,
                )
                .text("Speed"),
            );
        });
}
