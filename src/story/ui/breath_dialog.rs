use bevy::{prelude::*, utils::HashSet};
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::story::{
    ant::{hunger::Hunger, Dead},
    common::position::Position,
    element::{commands::ElementCommandsExt, Air, Element, Food},
    nest_simulation::nest::Nest,
};

use super::action_menu::IsShowingBreathDialog;

pub struct IsOpen(bool);

impl Default for IsOpen {
    fn default() -> Self {
        // default to true because need true for egui::Window to show "close" X icon
        Self(true)
    }
}

pub fn update_breath_dialog(
    mut contexts: EguiContexts,
    mut is_showing_breath_dialog: ResMut<IsShowingBreathDialog>,
    ant_query: Query<&Hunger, Without<Dead>>,
    food_query: Query<&Food>,
    air_query: Query<&Position, With<Air>>,
    mut rng: ResMut<GlobalRng>,
    nest: Res<Nest>,
    mut commands: Commands,
    mut is_running: Local<bool>,
    mut is_open: Local<IsOpen>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    let ctx = contexts.ctx_mut();

    let ant_count = ant_query.iter().len() as isize;
    // TODO: Should infer this from rate of hunger?
    let total_ant_food_needed = ant_count * 5;
    let food_count = food_query.iter().len();
    let ant_food_needed = isize::max(total_ant_food_needed as isize - food_count as isize, 0);
    let labels = ["Inhale", "Hold", "Exhale", "Hold"];
    let duration = 4.;

    if !is_open.0 {
        // TODO: eww. resetting this to true since this is resetting it to its default? :s
        is_open.0 = true;
        is_showing_breath_dialog.0 = false;
        *is_running = false;
        *timer = 0.;
    }

    egui::CentralPanel::default().show(ctx, |_| {
        egui::Window::new("Breathe")
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .resizable(false)
            .collapsible(false)
            .open(&mut is_open.0)
            .show(ctx, |ui| {
                if *is_running {
                    ui.vertical_centered(|ui| {
                        ui.columns(3, |columns: &mut [egui::Ui]| {
                            let current_index = ((*timer / duration) % duration) as usize;

                            columns[0].label("");
                            columns[0].vertical_centered(|ui| {
                                if current_index == 0 {
                                    ui.label(egui::RichText::new(labels[0]).strong());
                                } else {
                                    ui.label(egui::RichText::new(labels[0]).weak());
                                }
                            });
                            columns[0].label("");
                            columns[0].end_row();

                            columns[1].vertical_centered(|ui| {
                                if current_index == 1 {
                                    ui.label(egui::RichText::new(labels[1]).strong());
                                } else {
                                    ui.label(egui::RichText::new(labels[1]).weak());
                                }
                            });
                            columns[1].label("");
                            columns[1].vertical_centered(|ui| {
                                if current_index == 3 {
                                    ui.label(egui::RichText::new(labels[3]).strong());
                                } else {
                                    ui.label(egui::RichText::new(labels[3]).weak());
                                }
                            });
                            columns[1].end_row();

                            columns[2].label("");
                            columns[2].vertical_centered(|ui| {
                                if current_index == 2 {
                                    ui.label(egui::RichText::new(labels[2]).strong());
                                } else {
                                    ui.label(egui::RichText::new(labels[2]).weak());
                                }
                            });
                            columns[2].label("");
                            columns[2].end_row();
                        });

                        ui.separator();

                        // One cycle is 10 food
                        let ant_food_acquired =
                            ((timer.floor() / (labels.len() as f32 * duration)) as isize) * 10;

                        if ui
                            .button(&format!("Collect {:.0} Food", ant_food_acquired))
                            .clicked()
                        {
                            is_showing_breath_dialog.0 = false;
                            *is_running = false;
                            *timer = 0.;

                            let northern_air_positions = air_query
                                .iter()
                                .filter(|position| position.y < 20)
                                .collect::<Vec<&Position>>();

                            let mut spawn_positions: HashSet<Position> = HashSet::new();

                            while spawn_positions.len() < ant_food_acquired as usize {
                                let offset = rng.usize(0..northern_air_positions.len());
                                let position = northern_air_positions[offset];

                                spawn_positions.insert(*position);
                            }

                            for position in spawn_positions.iter() {
                                if let Some(entity) = nest.elements().get_element_entity(*position) {
                                    commands.replace_element(*position, Element::Food, *entity);
                                }
                            }
                        }
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label("Welcome.");
                        ui.label("Thanks for showing up.");
                        ui.separator();

                        ui.label(&format!(
                            "Today, you have {} ant{} and {} spare food.",
                            ant_count,
                            pluralize(ant_count),
                            food_count
                        ));

                        ui.label(&format!(
                            "You need {} more food to keep your ant{} fed.",
                            ant_food_needed,
                            pluralize(ant_count)
                        ));

                        let seconds =
                            (ant_food_needed / 10) * (duration as isize * labels.len() as isize);
                        ui.label(&format!(
                            "That's about {} second{} of breathing.",
                            seconds,
                            pluralize(seconds)
                        ));

                        ui.separator();

                        ui.label("This time is for you.");
                        ui.label("Enjoy.");

                        if ui.button("Begin").clicked() {
                            *is_running = true;
                        }
                    });
                }
            });
    });

    if *is_running {
        *timer += time.delta_seconds();
    }
}

fn pluralize(value: isize) -> &'static str {
    if value != 1 {
        "s"
    } else {
        ""
    }
}
