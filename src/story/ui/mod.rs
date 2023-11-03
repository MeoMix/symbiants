pub mod action_menu;
mod breath_dialog;
pub mod selection_menu;
mod settings_menu;

mod info_panel;
mod loading_dialog;
mod story_over_dialog;

use crate::story::story_time::StoryPlaybackState;
use crate::app_state::AppState;

use self::action_menu::*;
use self::breath_dialog::update_breath_dialog;
use self::info_panel::*;
use self::loading_dialog::*;
use self::selection_menu::update_selection_menu;
use self::settings_menu::update_settings_menu;
use self::story_over_dialog::*;
use bevy::prelude::*;

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
