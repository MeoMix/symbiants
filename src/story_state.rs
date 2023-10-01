use bevy::prelude::*;

use crate::{
    ant::{AntRole, Dead},
    world_map::save::delete_save,
};

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
    GatheringSettings,
    Creating,
    FinalizingStartup,

    Telling,
    Over,
    Cleanup,
}

pub fn on_story_cleanup(mut story_state: ResMut<NextState<StoryState>>) {
    delete_save();
    story_state.set(StoryState::Initializing);
}

pub fn check_story_over(
    dead_ants_query: Query<&AntRole, With<Dead>>,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    if dead_ants_query
        .iter()
        .any(|ant_role| *ant_role == AntRole::Queen)
    {
        story_state.set(StoryState::Over);
    }
}
