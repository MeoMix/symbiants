// TODO: It's weird this needs to be public
pub mod action_menu;
mod breath_dialog;
mod info_panel;
mod loading_dialog;
mod selection_menu;
mod settings_menu;
mod story_over_dialog;

use self::{
    action_menu::*, breath_dialog::update_breath_dialog, info_panel::*, loading_dialog::*,
    selection_menu::update_selection_menu, settings_menu::update_settings_menu,
    story_over_dialog::*,
};
use bevy::prelude::*;
use simulation::{app_state::AppState, story_time::StoryPlaybackState};

pub struct StoryUIPlugin;

impl Plugin for StoryUIPlugin {
    fn build(&self, app: &mut App) {
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
}
