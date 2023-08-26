use bevy::prelude::*;

use crate::{ant::{AntRole, Dead}, grid::save::delete_save};

// TODO: Probably split this into AppState and StoryState where AppState encompasses the app
// and StoryState is a single instance, usually 1:1 but 0:1 during story creation.

// NOTE: I don't think there's a way to persist this nor should it be persisted - seems like it's useful for controlling view state?
// So I initialize it from model state in setup_story_state
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

pub fn setup_story_state(
    mut story_state: ResMut<NextState<StoryState>>,
    dead_ants_query: Query<&AntRole, With<Dead>>,
) {
    if dead_ants_query
        .iter()
        .any(|ant_role| *ant_role == AntRole::Queen)
    {
        story_state.set(StoryState::Over);
    }
}

pub fn on_story_cleanup(mut story_state: ResMut<NextState<StoryState>>) {
    delete_save();

    // TODO:: I think we could skip going to Initializing and go to GatheringSettings
    // and also avoid deleting the save -- but nothing wrong with doing these things either
    // it just ends up checking local storage again when we know it won't be there
    story_state.set(StoryState::Initializing);
}
