use bevy::prelude::*;

use crate::story::ant::{AntRole, Dead};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    BeginSetup,
    TryLoadSave,
    ShowMainMenu,
    CreateNewStory,
    FinishSetup,
    TellStory,
    EndStory,
    Cleanup,
}

pub fn restart(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::BeginSetup);
}

pub fn continue_startup(
    In(is_loading_existing_story): In<bool>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if is_loading_existing_story {
        next_app_state.set(AppState::FinishSetup);
    } else {
        next_app_state.set(AppState::ShowMainMenu);
    }
}

pub fn finalize_startup(mut next_app_state: ResMut<NextState<AppState>>) {
    next_app_state.set(AppState::FinishSetup);
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
