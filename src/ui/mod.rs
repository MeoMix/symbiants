pub mod action_menu;
mod main_menu;
mod settings_menu;
mod story;

use crate::story_state::StoryState;

use self::action_menu::*;
use self::main_menu::*;
use self::settings_menu::update_settings_menu;
use self::story::info_panel::*;
use self::story::loading_dialog::*;
use self::story::story_over_dialog::*;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);

        app.add_systems(
            Update,
            update_main_menu_dialog.run_if(in_state(StoryState::GatheringSettings)),
        );

        app.add_systems(OnEnter(StoryState::Telling), initialize_action_menu);

        // TODO: Prefer keeping UI around until after Over (but not that simple because can click Reset which skips Over)
        app.add_systems(
            Update,
            (
                update_info_window,
                update_loading_dialog,
                update_settings_menu,
                update_action_menu,
            )
                .run_if(in_state(StoryState::Telling)),
        );

        app.add_systems(OnExit(StoryState::Telling), deinitialize_action_menu);

        app.add_systems(
            Update,
            update_story_over_dialog.run_if(in_state(StoryState::Over)),
        );
    }
}
