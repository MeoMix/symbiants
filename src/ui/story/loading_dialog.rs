use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story_time::{FastForwardingStateInfo, StoryPlaybackState, TicksPerSecond};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 6000;

pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    fast_forwarding_state_info: Res<FastForwardingStateInfo>,
    story_playback_state: ResMut<State<StoryPlaybackState>>,
    ticks_per_second: Res<TicksPerSecond>,
) {
    if story_playback_state.get() != &StoryPlaybackState::FastForwarding
        || fast_forwarding_state_info.initial_pending_ticks < MIN_PENDING_TICKS
    {
        return;
    }

    egui::Window::new("Loading")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            let seconds_gone = (fast_forwarding_state_info.initial_pending_ticks as f32)
                / ticks_per_second.0 as f32;

            if seconds_gone > 60.0 {
                let minutes_gone = seconds_gone / 60.0;
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
