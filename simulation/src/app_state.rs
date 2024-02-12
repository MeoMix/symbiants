use bevy::prelude::*;

use crate::{
    common::ant::{AntRole, Dead}, story_time::StoryPlaybackState
};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    BeginSetup,
    TryLoadSave,
    SelectStoryMode,
    CreateNewStory,
    FinishSetup,
    // Bevy does not currently support adding systems at runtime. So, systems
    // which monitor for Added<_> have a backlog to process, but this is not desirable
    // as they are intended for a running simulation not an initializing simulation. 
    PostSetupClearChangeDetection,
    TellStory,
    EndStory,
    Cleanup,
}

pub fn restart(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_story_playback_state: ResMut<NextState<StoryPlaybackState>>,
) {
    next_story_playback_state.set(StoryPlaybackState::Stopped);
    next_app_state.set(AppState::BeginSetup);
}

pub fn continue_startup(
    In(is_loading_existing_story): In<bool>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if is_loading_existing_story {
        next_app_state.set(AppState::FinishSetup);
    } else {
        next_app_state.set(AppState::SelectStoryMode);
    }
}

pub fn finalize_startup(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::FinishSetup);
}

pub fn post_setup_clear_change_detection(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::PostSetupClearChangeDetection);
}

pub fn begin_story(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::TellStory);
}

pub fn check_story_over(
    dead_ants_query: Query<&AntRole, With<Dead>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if dead_ants_query
        .iter()
        .any(|ant_role| *ant_role == AntRole::Queen)
    {
        next_app_state.set(AppState::EndStory);
    }
}
