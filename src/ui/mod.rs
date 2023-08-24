mod common;
mod command_buttons;
mod info_panel;
mod loading_dialog;
mod story_over_dialog;

use crate::story_state::StoryState;

use self::common::button::*;
use self::command_buttons::*;
use self::info_panel::*;
use self::loading_dialog::*;
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
            ),
        );
        
        app.add_systems(
            Update,
            handle_reset_button_interaction,
        );

        app.add_systems(OnEnter(StoryState::Over), setup_story_over_dialog);
        app.add_systems(
            OnExit(StoryState::Over),
            despawn_screen::<StoryOverDialogModalOverlay>,
        );

        app.add_systems(
            Update,
            handle_story_over_dialog_button_interactions.run_if(in_state(StoryState::Over)),
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
