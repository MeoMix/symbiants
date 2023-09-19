use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    ant::{hunger::Hunger, AntRole},
    element::Food,
};

pub fn update_info_window(
    mut contexts: EguiContexts,
    ant_query: Query<(&AntRole, &Hunger)>,
    food_query: Query<&Food>,
) {
    let queen_ant = ant_query.iter().find(|(&role, _)| role == AntRole::Queen);
    let queen_ant_hunger = queen_ant.map(|(_, hunger)| hunger.value()).unwrap_or(0.0);

    egui::Window::new("Info")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(&format!("Ants: {}", ant_query.iter().count()));
            ui.label(&format!("Queen Hunger: {:.0}%", queen_ant_hunger));
            ui.label(&format!("Food: {}", food_query.iter().count()));
        });
}
