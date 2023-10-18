use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    ant::{birthing::Birthing, hunger::Hunger, AntRole},
    element::Food,
    story_time::{StoryElapsedTicks, DEFAULT_TICKS_PER_SECOND, SECONDS_PER_DAY, SECONDS_PER_HOUR},
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

    egui::Window::new("Info")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let seconds_total = elapsed_ticks.0 as f32 / DEFAULT_TICKS_PER_SECOND as f32;
            let days = seconds_total / SECONDS_PER_DAY as f32;

            // Calculate hours and minutes
            let hours_total = (seconds_total % SECONDS_PER_DAY as f32) / SECONDS_PER_HOUR as f32;
            let hours = hours_total.floor();
            let minutes = ((hours_total - hours) * 60.0).floor();

            // Determine AM/PM and adjust hour to 12-hour format
            let (period, hour_12) = if hours < 12.0 {
                ("AM", if hours == 0.0 { 12.0 } else { hours })
            } else {
                ("PM", if hours > 12.0 { hours - 12.0 } else { hours })
            };

            // Construct the label string
            ui.label(&format!(
                "Day: {:.0}, {:02.0}:{:02.0} {}",
                days.floor(),
                hour_12,
                minutes,
                period
            ));

            ui.label(&format!("Ants: {}", ant_query.iter().count()));
            ui.label(&format!("Queen Hunger: {:.0}%", queen_ant_hunger));
            ui.label(&format!("Queen Birthing: {:.0}%", queen_ant_birthing));
            ui.label(&format!("Food: {}", food_query.iter().count()));
        });
}
