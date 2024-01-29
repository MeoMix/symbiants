use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::{
    nest_simulation::{
        ant::{birthing::Birthing, hunger::Hunger, AntRole, Dead},
        element::Food,
    },
    story_time::StoryTime,
};

pub fn update_info_window(
    mut contexts: EguiContexts,
    ant_query: Query<(&AntRole, &Hunger, Option<&Birthing>), Without<Dead>>,
    food_query: Query<&Food>,
    story_time: Res<StoryTime>,
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
    let colony_average_hunger = ant_query
        .iter()
        .fold(0.0, |acc, (_, hunger, _)| acc + hunger.value())
        / ant_query.iter().count() as f32;

    egui::Window::new("Info")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let time_info = story_time.as_time_info();

            // Determine AM/PM and adjust hour to 12-hour format
            let hours = time_info.hours();
            let (period, hour_12) = if hours < 12 {
                ("AM", if hours == 0 { 12 } else { hours })
            } else {
                ("PM", if hours > 12 { hours - 12 } else { hours })
            };

            // Construct the label string
            ui.label(&format!(
                "Day: {:.0}, {:02.0}:{:02.0} {}",
                // Add one to the days label because days don't start at 0 in real life
                time_info.days() + 1,
                hour_12,
                time_info.minutes(),
                period
            ));

            ui.label(&format!("Alive Ants: {}", ant_query.iter().count()));
            ui.label(&format!(
                "Colony Average Hunger: {:.0}%",
                colony_average_hunger
            ));
            ui.label(&format!("Queen Hunger: {:.0}%", queen_ant_hunger));
            ui.label(&format!("Queen Birthing: {:.0}%", queen_ant_birthing));
            ui.label(&format!("Food: {}", food_query.iter().count()));
        });
}
