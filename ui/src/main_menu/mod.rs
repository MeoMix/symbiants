use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use simulation::app_state::AppState;

pub struct MainMenuUIPlugin;

impl Plugin for MainMenuUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_main_menu.run_if(in_state(AppState::MainMenu)),
        );
    }
}

pub fn update_main_menu(
    mut contexts: EguiContexts,
    mut next_app_state: ResMut<NextState<AppState>>,
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
                    next_app_state.set(AppState::FinishSetup);
                }
            });
        });
}
