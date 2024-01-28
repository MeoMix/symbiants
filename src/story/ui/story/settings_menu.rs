use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use simulation::{
    app_state::AppState,
    nest_simulation::{ant::AntColor, pheromone::PheromoneVisibility},
    settings::Settings,
    story_time::{
        StoryPlaybackState, StoryTime, TicksPerSecond, DEFAULT_TICKS_PER_SECOND,
        MAX_USER_TICKS_PER_SECOND,
    },
};

pub fn update_settings_menu(
    mut contexts: EguiContexts,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut ticks_per_second: ResMut<TicksPerSecond>,
    story_playback_state: Res<State<StoryPlaybackState>>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
    mut pheromone_visibility: ResMut<PheromoneVisibility>,
    mut story_time: ResMut<StoryTime>,
    mut settings: ResMut<Settings>,
    mut ant_query: Query<&mut AntColor>,
) {
    let window = primary_window_query.single();
    let ctx = contexts.ctx_mut();

    egui::Window::new("Settings")
        .default_pos(egui::Pos2::new(window.width() - 400.0, 0.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.checkbox(&mut story_time.is_real_time, "Use Real Time");

            ui.add_enabled_ui(story_time.is_real_time, |ui| {
                ui.checkbox(&mut story_time.is_real_sun, "Use Real Sunrise/Sunset");
            });

            ui.add_enabled_ui(story_time.is_real_sun, |ui| {
                ui.horizontal_top(|ui| {
                    // TODO: egui doesn't support numeric inputs
                    // https://github.com/emilk/egui/issues/1348
                    // TODO: introduce separate (local?) state tracking for input to support temporarily incorrect state.
                    // This will enable supporting writing "1.05" directly rather than needing to say "105" then go back and add the period.
                    let mut temp_latitude = story_time.latitude.to_string();
                    let latitude_input = egui::TextEdit::singleline(&mut temp_latitude);

                    ui.label("Lat.");
                    ui.add(
                        latitude_input
                            .desired_width(50.0)
                            .interactive(story_time.is_real_sun),
                    );

                    // attempt to parse latitude string back into float:
                    if let Ok(new_latitude) = temp_latitude.parse::<f32>() {
                        story_time.latitude = new_latitude;
                    }

                    let mut temp_longitude = story_time.longitude.to_string();
                    let longitude_input = egui::TextEdit::singleline(&mut temp_longitude);
                    ui.label("Long.");
                    ui.add(
                        longitude_input
                            .desired_width(50.0)
                            .interactive(story_time.is_real_sun),
                    );

                    if let Ok(new_longitude) = temp_longitude.parse::<f32>() {
                        story_time.longitude = new_longitude;
                    }
                });
            });

            ui.add_enabled_ui(story_time.is_real_time, |ui| {
                ui.checkbox(
                    &mut settings.is_breathwork_scheduled,
                    "Use Breathwork Scheduling",
                );

                let (sunrise, _) = story_time.get_sunrise_sunset_decimal_hours();

                let (hours, minutes) = decimal_hours_to_hours_minutes(sunrise);

                ui.label(&format!("Unlock Time: {}:{:02} AM", hours - 2.0, minutes));
                ui.label(&format!("Lock Time: {}:{:02} AM", hours + 2.0, minutes));
            });

            ui.add(
                egui::Slider::new(
                    &mut ticks_per_second.0,
                    DEFAULT_TICKS_PER_SECOND..=MAX_USER_TICKS_PER_SECOND,
                )
                .text("ticks/sec"),
            );

            match story_playback_state.get() {
                StoryPlaybackState::Playing => {
                    if ui.button("Pause").clicked() {
                        next_story_playback_state.set(StoryPlaybackState::Paused);
                    }
                }
                StoryPlaybackState::Paused => {
                    if ui.button("Play").clicked() {
                        next_story_playback_state.set(StoryPlaybackState::Playing);
                    }
                }
                StoryPlaybackState::FastForwarding => {
                    ui.add_enabled(false, egui::Button::new("Fast Forwarding"));
                }
                StoryPlaybackState::Stopped => {
                    ui.add_enabled(false, egui::Button::new("Stopped"));
                }
            }

            if pheromone_visibility.0 == Visibility::Hidden {
                if ui.button("Show Pheromones").clicked() {
                    pheromone_visibility.0 = Visibility::Visible;
                }
            } else if pheromone_visibility.0 == Visibility::Visible {
                if ui.button("Hide Pheromones").clicked() {
                    pheromone_visibility.0 = Visibility::Hidden;
                }
            }

            ui.horizontal_top(|ui| {
                ui.label("Ant Color");

                let mut egui_color = bevy_color_to_color32(settings.ant_color);
                egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut egui_color,
                    egui::color_picker::Alpha::OnlyBlend,
                );
                let new_ant_color = color32_to_bevy_color(egui_color);

                if settings.ant_color != new_ant_color {
                    settings.ant_color = new_ant_color;

                    for mut ant_color in ant_query.iter_mut() {
                        ant_color.0 = new_ant_color;
                    }
                }
            });

            if ui.button("Reset Sandbox").clicked() {
                next_app_state.set(AppState::Cleanup);
            }
        });
}

fn color32_to_bevy_color(color: egui::Color32) -> bevy::prelude::Color {
    bevy::prelude::Color::rgba(
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
        color.a() as f32 / 255.0,
    )
}

fn bevy_color_to_color32(color: bevy::prelude::Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color.r() * 255.0) as u8,
        (color.g() * 255.0) as u8,
        (color.b() * 255.0) as u8,
        (color.a() * 255.0) as u8,
    )
}

fn decimal_hours_to_hours_minutes(decimal_hours: f32) -> (f32, f32) {
    let hours = decimal_hours.trunc();
    let minutes = (decimal_hours.fract() * 60.0).round();
    (hours, minutes)
}
