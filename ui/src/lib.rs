mod main_menu;
pub mod story;

use self::{main_menu::MainMenuUIPlugin, story::StoryUIPlugin};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, TextStyle},
    EguiContexts, EguiPlugin,
};
use egui::{FontFamily::Proportional, FontId};
use rendering::common::pointer::IsPointerCaptured;
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: There is no Camera spawned here because UI is built using `bevy_egui` which doesn't care about cameras.
        // In the future, if UI is rebuilt using Bevy natively, then a UI camera may need to be spawned here.
        app.add_plugins((EguiPlugin, MainMenuUIPlugin, StoryUIPlugin));

        #[cfg(feature = "dev-inspector")]
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

        app.add_systems(Update, set_theme);

        app.add_systems(
            PostUpdate,
            is_pointer_captured.run_if(resource_exists::<IsPointerCaptured>),
        );
    }
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
    style.visuals.window_highlight_topmost = false;

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

pub fn is_pointer_captured(
    mut is_pointer_captured: ResMut<IsPointerCaptured>,
    mut contexts: EguiContexts,
) {
    let context = contexts.ctx_mut();
    is_pointer_captured.0 = context.wants_pointer_input() || context.wants_keyboard_input();
}
