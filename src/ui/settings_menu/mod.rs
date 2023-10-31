use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{
    ant::AntColor,
    pheromone::PheromoneVisibility,
    settings::Settings,
    story_state::StoryState,
    story_time::{
        StoryPlaybackState, StoryTime, TicksPerSecond, DEFAULT_TICKS_PER_SECOND,
        MAX_USER_TICKS_PER_SECOND,
    },
};

pub fn update_settings_menu(
    mut contexts: EguiContexts,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut next_story_state: ResMut<NextState<StoryState>>,
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

            //ui.checkbox(&mut story_time.is_real_sun, "Use Real Sunrise/Sunset");

            // TODO: egui doesn't support numeric inputs
            // https://github.com/emilk/egui/issues/1348
            // lat/long
            // ui.add(egui::TextEdit::singleline(&mut story_time.latitude).hint_text("Lat"));
            // ui.add(egui::TextEdit::singleline(&mut story_time.longitutde).hint_text("Long"));

            ui.add(
                egui::Slider::new(
                    &mut ticks_per_second.0,
                    DEFAULT_TICKS_PER_SECOND..=MAX_USER_TICKS_PER_SECOND,
                )
                .text("Speed"),
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

            if ui.button("Reset Story").clicked() {
                next_story_state.set(StoryState::Cleanup);
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
