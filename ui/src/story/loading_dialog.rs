use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use simulation::story_time::{FastForwardPendingTicks, TicksPerSecond, SECONDS_PER_DAY};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 6000;

pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    fast_forward_pending_ticks: Res<FastForwardPendingTicks>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    if fast_forward_pending_ticks.initial() < MIN_PENDING_TICKS {
        return;
    }

    egui::Window::new("Loading")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let seconds_gone = fast_forward_pending_ticks.initial()
                / ticks_per_second.0;
            let minutes_gone = seconds_gone / 60;
            let hours_gone = minutes_gone / 60;
            let minutes_remaining = minutes_gone - (hours_gone * 60);

            if hours_gone > 0 {
                if minutes_remaining > 0 {
                    ui.label(&format!(
                        "You were gone for {:.0} hour{} and {:.0} minute{}.",
                        hours_gone,
                        pluralize(hours_gone),
                        minutes_remaining,
                        pluralize(minutes_remaining)
                    ));
                } else {
                    ui.label(&format!(
                        "You were gone for {:.0} hour{}.",
                        hours_gone,
                        pluralize(hours_gone),
                    ));
                }
            } else if minutes_gone > 0 {
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

            ui.label("Please wait while this time is simulated.");

            if seconds_gone == SECONDS_PER_DAY {
                ui.label("NOTE: Simulation will only fast-forward through one day of missed time. Please check in daily.");
            }

            ui.label(&format!(
                "Remaining ticks: {}",
                fast_forward_pending_ticks.remaining()
            ));
        });
}

fn pluralize(value: isize) -> &'static str {
    if value != 1 {
        "s"
    } else {
        ""
    }
}
