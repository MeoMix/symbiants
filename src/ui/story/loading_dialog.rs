use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts,
};

use crate::story_time::{FastForwardingStateInfo, StoryPlaybackState, DEFAULT_TICKS_PER_SECOND};

// Don't flicker the dialogs visibility when processing a small number of ticks
const MIN_PENDING_TICKS: isize = 6000;

pub fn update_loading_dialog(
    mut contexts: EguiContexts,
    fast_forwarding_state_info: Res<FastForwardingStateInfo>,
    story_playback_state: ResMut<State<StoryPlaybackState>>,
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
            let minutes_gone = (fast_forwarding_state_info.initial_pending_ticks as f32)
                / (60.0 / DEFAULT_TICKS_PER_SECOND * 60.0);

            ui.label(&format!("You were gone for {:.0} minutes.", minutes_gone));
            ui.label("Please wait while this time is simulated.");
            ui.label(&format!(
                "Remaining ticks: {}",
                fast_forwarding_state_info.pending_ticks
            ));
        });
}
