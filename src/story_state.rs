use bevy::prelude::*;

use crate::ant::{AntRole, Dead};

// NOTE: I don't think there's a way to persist this nor should it be persisted - seems like it's useful for controlling view state?
// So I initialize it from model state in setup_story_state
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum StoryState {
    #[default]
    NotStarted,
    Telling,
    Over,
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
