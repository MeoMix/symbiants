// TODO: It's weird this needs to be public
pub mod action_menu;
mod breath_dialog;
mod info_panel;
mod loading_dialog;
mod main_menu;
mod selection_menu;
mod settings_menu;
mod story_over_dialog;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextStyle},
    EguiContexts, EguiPlugin,
};
use egui::{FontFamily::Proportional, FontId};

use super::pointer::IsPointerCaptured;
use simulation::{app_state::AppState, story_time::StoryPlaybackState};

use self::{
    action_menu::*, breath_dialog::update_breath_dialog, info_panel::*, loading_dialog::*,
    main_menu::update_main_menu, selection_menu::update_selection_menu,
    settings_menu::update_settings_menu, story_over_dialog::*,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);
        app.init_resource::<IsPointerCaptured>();
        app.add_systems(Update, set_theme);

        build_story_systems(app);
        build_main_menu_systems(app);
    }
}

fn build_main_menu_systems(app: &mut App) {
    app.add_systems(
        Update,
        update_main_menu.run_if(in_state(AppState::SelectStoryMode)),
    );
}

fn build_story_systems(app: &mut App) {
    app.add_systems(OnEnter(AppState::TellStory), setup_action_menu);

    // TODO: Prefer keeping UI around until after Over (but not that simple because can click Reset which skips Over)
    app.add_systems(
        Update,
        (
            update_info_window,
            update_loading_dialog.run_if(in_state(StoryPlaybackState::FastForwarding)),
            update_settings_menu,
            update_action_menu,
            update_selection_menu,
        )
            .run_if(
                in_state(AppState::TellStory)
                    .and_then(not(resource_exists_and_equals(IsShowingBreathDialog(true)))),
            ),
    );

    app.add_systems(
        Update,
        update_breath_dialog.run_if(resource_exists_and_equals(IsShowingBreathDialog(true))),
    );

    app.add_systems(OnExit(AppState::TellStory), teardown_action_menu);

    app.add_systems(
        Update,
        update_story_over_dialog.run_if(
            in_state(AppState::EndStory)
                .and_then(not(resource_exists_and_equals(IsShowingBreathDialog(true)))),
        ),
    );
}

/// This themeing isn't good by any means, but it serves as an example for how to adjust it further. It would be nice to have it look much more like Material UI
fn set_theme(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    // TODO: This is just a very temporary hack to make iOS UI look better/usable.
    // I think that the user probably expects the UI to scale with zoom in/out and that'll take more fussing.
    let mut scale_factor = 1.0;
    if ctx.pixels_per_point() >= 2.0 {
        scale_factor = 1.5;
    }

    style.spacing.window_margin = egui::Margin::symmetric(12.0, 18.0);
    style.spacing.item_spacing = egui::Vec2::new(8.0, 12.0);
    style.spacing.button_padding = egui::Vec2::new(8.0, 8.0);

    style.visuals.window_fill = egui::Color32::from_black_alpha(224);

    // Redefine text_styles
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(20.0 * scale_factor, Proportional),
        ),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(18.0 * scale_factor, Proportional),
        ),
        (
            TextStyle::Name("Context".into()),
            FontId::new(18.0 * scale_factor, Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(14.0 * scale_factor, Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(14.0 * scale_factor, Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(10.0 * scale_factor, Proportional),
        ),
    ]
    .into();

    // Mutate global style with above changes
    ctx.set_style(style);
}
