use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    ant::{birthing::Birthing, hunger::Hunger, AntRole},
    element::Food,
    story_time::StoryElapsedTicks,
};

pub fn update_info_window(
    mut contexts: EguiContexts,
    ant_query: Query<(&AntRole, &Hunger, Option<&Birthing>)>,
    food_query: Query<&Food>,
    elapsed_ticks: Res<StoryElapsedTicks>,
) {
    let queen_ant = ant_query
        .iter()
        .find(|(&role, _, _)| role == AntRole::Queen);
    let queen_ant_hunger = queen_ant
        .map(|(_, hunger, _)| hunger.value())
        .unwrap_or(0.0);
    let queen_ant_birthing = queen_ant
        .map(|(_, _, birthing_option)| birthing_option.map_or(0.0, |birthing| birthing.value()))
        .unwrap_or(0.0);
    let colony_average_hunger = ant_query.iter().fold(0.0, |acc, (_, hunger, _)| {
        acc + hunger.value()
    }) / ant_query.iter().count() as f32; 

    egui::Window::new("Info")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let time_info = elapsed_ticks.as_time_info();

            // Determine AM/PM and adjust hour to 12-hour format
            let hours = time_info.hours;
            let (period, hour_12) = if hours < 12 {
                ("AM", if hours == 0 { 12 } else { hours })
            } else {
                ("PM", if hours > 12 { hours - 12 } else { hours })
            };

            // Construct the label string
            ui.label(&format!(
                "Day: {:.0}, {:02.0}:{:02.0} {}",
                time_info.days, hour_12, time_info.minutes, period
            ));

            ui.label(&format!("Ants: {}", ant_query.iter().count()));
            ui.label(&format!("Colony Average Hunger: {:.0}%", colony_average_hunger));
            ui.label(&format!("Queen Hunger: {:.0}%", queen_ant_hunger));
            ui.label(&format!("Queen Birthing: {:.0}%", queen_ant_birthing));
            ui.label(&format!("Food: {}", food_query.iter().count()));
        });
}
