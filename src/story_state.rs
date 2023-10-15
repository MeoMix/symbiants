use bevy::prelude::*;

use crate::ant::{AntRole, Dead};

// TODO: Probably split this into AppState and StoryState where AppState encompasses the app
// and StoryState is a single instance, usually 1:1 but 0:1 during story creation.

// STEPS:
// 1) Load everything that is needed for multiple stories.
// 2) Check save state.
// 3) If save state exists, load saved story
// 4) If save state does not exist, show main menu.
// 5) If user creates new story from Main Menu then create new story.
// 6) Load everything needed for new story.
// 7) Load everything needed for current story.
// 8) Tell story
// 9) Mark story over
// 10) Cleanup story

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum StoryState {
    #[default]
    Initializing,
    LoadingSave,
    GatheringSettings,
    Creating,
    FinalizingStartup,
    Telling,
    Over,
    Cleanup,
}

pub fn restart_story(mut next_story_state: ResMut<NextState<StoryState>>) {
    next_story_state.set(StoryState::Initializing);
}

pub fn continue_startup(
    In(is_loading_existing_story): In<bool>,
    mut next_story_state: ResMut<NextState<StoryState>>,
) {
    if is_loading_existing_story {
        next_story_state.set(StoryState::FinalizingStartup);
    } else {
        next_story_state.set(StoryState::GatheringSettings);
    }
}

pub fn finalize_startup(mut next_story_state: ResMut<NextState<StoryState>>) {
    next_story_state.set(StoryState::FinalizingStartup);
}

pub fn begin_story(mut next_story_state: ResMut<NextState<StoryState>>) {
    next_story_state.set(StoryState::Telling);
}

pub fn check_story_over(
    dead_ants_query: Query<&AntRole, With<Dead>>,
    mut next_story_state: ResMut<NextState<StoryState>>,
) {
    if dead_ants_query
        .iter()
        .any(|ant_role| *ant_role == AntRole::Queen)
    {
        next_story_state.set(StoryState::Over);
    }
}
