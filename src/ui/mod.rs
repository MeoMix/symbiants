mod command_buttons;
mod common;
mod info_panel;
mod loading_dialog;
mod main_menu_dialog;
mod story_over_dialog;

use crate::story_state::StoryState;

use self::command_buttons::*;
use self::common::button::*;
use self::info_panel::*;
use self::loading_dialog::*;
use self::main_menu_dialog::*;
use self::story_over_dialog::*;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_info_panel, setup_command_buttons));
        app.add_systems(
            Update,
            (
                update_loading_dialog,
                update_info_panel_ant_count,
                update_info_panel_ant_hunger,
                update_info_panel_food,
                update_info_panel_day,
                update_food_button,
            )
                .run_if(in_state(StoryState::Telling)),
        );

        app.add_systems(Update, handle_reset_button_interaction);

        app.add_systems(OnEnter(StoryState::GatheringSettings), setup_main_menu_dialog);
        app.add_systems(
            OnExit(StoryState::GatheringSettings),
            despawn_screen::<MainMenuDialogModalOverlay>,
        );

        app.add_systems(
            Update,
            on_interact_main_menu_button.run_if(in_state(StoryState::GatheringSettings)),
        );

        app.add_systems(OnEnter(StoryState::Over), setup_story_over_dialog);
        app.add_systems(
            OnExit(StoryState::Over),
            despawn_screen::<StoryOverDialogModalOverlay>,
        );

        app.add_systems(
            Update,
            on_interact_story_over_button.run_if(in_state(StoryState::Over)),
        );

        app.add_systems(Update, button_system);
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
