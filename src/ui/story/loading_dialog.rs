use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::time::{IsFastForwarding, PendingTicks};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 1000;

// TODO: modal overlay
pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    pending_ticks: Res<PendingTicks>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    if !is_fast_forwarding.0 || pending_ticks.0 < MIN_PENDING_TICKS {
        return;
    }

    egui::Window::new("Loading")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(&format!("You were gone for {:.0} minutes.", pending_ticks.as_minutes()));
            ui.label("Please wait while this time is simulated.");
            ui.label(&format!("Remaining ticks: {}", pending_ticks.0));
        });
}
