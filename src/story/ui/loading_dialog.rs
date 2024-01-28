use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story::simulation::story_time::{
    FastForwardingStateInfo, TicksPerSecond, SECONDS_PER_DAY,
};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 6000;

pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    fast_forwarding_state_info: Res<FastForwardingStateInfo>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    if fast_forwarding_state_info.initial_pending_ticks < MIN_PENDING_TICKS {
        return;
    }

    egui::Window::new("Loading")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let seconds_gone = (fast_forwarding_state_info.initial_pending_ticks as f32)
                / ticks_per_second.0 as f32;

            let minutes_gone = seconds_gone / 60.0;
            let hours_gone = minutes_gone / 60.0;

            if hours_gone >= 1.0 {
                ui.label(&format!(
                    "You were gone for {:.0} hour{} and {:.0} minute{}.",
                    hours_gone,
                    pluralize(hours_gone / 60.0),
                    minutes_gone / 60.0,
                    pluralize(minutes_gone / 60.0)
                ));
            } else if minutes_gone >= 1.0 {
                ui.label(&format!(
                    "You were gone for {:.0} minute{}.",
                    minutes_gone,
                    pluralize(minutes_gone)
                ));
            } else {
                ui.label(&format!(
                    "You were gone for {:.0} second{}.",
                    seconds_gone,
                    pluralize(seconds_gone)
                ));
            }

            if seconds_gone >= SECONDS_PER_DAY as f32 {
                ui.label("Please wait while 24 hours are simulated.");
                ui.label("IMPORTANT: Your Symbiant simulation stops if you don't check in daily.");
            } else {
                ui.label("Please wait while this time is simulated.");
            }

            ui.label(&format!(
                "Remaining ticks: {}",
                fast_forwarding_state_info.pending_ticks
            ));
        });
}

fn pluralize(value: f32) -> &'static str {
    if value != 1.0 {
        "s"
    } else {
        ""
    }
}
