mod info_panel;
mod loading_dialog;
mod food_button;

use bevy::prelude::*;
use self::info_panel::*;
use self::loading_dialog::*;
use self::food_button::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_loading_text, setup_info_panel, setup_food_button));
        app.add_systems(Update, (update_loading_text, update_info_panel_ant_count, update_info_panel_ant_hunger, update_info_panel_food, update_info_panel_day, update_food_button));

    }
}
