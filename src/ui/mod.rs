pub mod action_menu;
mod common;
mod main_menu;
mod settings_menu;
mod story;

use crate::story_state::StoryState;

use self::action_menu::*;
use self::common::button::*;
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

        // Common:
        app.add_systems(Update, button_system);

        // Main Menu:
        app.add_systems(
            OnEnter(StoryState::GatheringSettings),
            create_main_menu_dialog,
        );
        app.add_systems(
            Update,
            on_interact_main_menu_button.run_if(in_state(StoryState::GatheringSettings)),
        );
        app.add_systems(
            OnExit(StoryState::GatheringSettings),
            despawn_screen::<MainMenuDialogModalOverlay>,
        );

        // TODO: Prefer keeping UI around until after Over (but not that simple because can click Reset which skips Over)
        // Story:
        app.add_systems(
            OnEnter(StoryState::Telling),
            (
                setup_info_panel,
                initialize_action_menu,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                update_loading_dialog,
                update_info_panel_ant_count,
                update_info_panel_ant_hunger,
                update_info_panel_food,
                update_settings_menu,
                update_action_menu,
            )
                .run_if(in_state(StoryState::Telling)),
        );

        app.add_systems(
            OnExit(StoryState::Telling),
            (
                despawn_screen::<InfoPanel>,
                deinitialize_action_menu,
            ),
        );

        app.add_systems(OnEnter(StoryState::Over), setup_story_over_dialog);
        app.add_systems(
            Update,
            on_interact_story_over_button.run_if(in_state(StoryState::Over)),
        );
        app.add_systems(
            OnExit(StoryState::Over),
            despawn_screen::<StoryOverDialogModalOverlay>,
        );
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
