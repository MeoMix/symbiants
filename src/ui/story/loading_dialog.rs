use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story_time::{IsFastForwarding, TotalPendingTicks, RemainingPendingTicks, TicksPerSecond};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 1000;

pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    remaining_pending_ticks: Res<RemainingPendingTicks>,
    total_pending_ticks: Res<TotalPendingTicks>,
    is_fast_forwarding: Res<IsFastForwarding>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    if !is_fast_forwarding.0 || total_pending_ticks.0 < MIN_PENDING_TICKS {
        return;
    }

    egui::Window::new("Loading")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let minutes_gone = (total_pending_ticks.0 as f32) / (60.0 / ticks_per_second.0 * 60.0);

            ui.label(&format!("You were gone for {:.0} minutes.", minutes_gone));
            ui.label("Please wait while this time is simulated.");
            ui.label(&format!("Remaining ticks: {}", remaining_pending_ticks.0));
        });
}
