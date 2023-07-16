mod info_panel;
mod loading;

use bevy::prelude::*;
use self::info_panel::*;
use self::loading::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_loading_text, setup_info_panel));
        app.add_systems(Update, (update_loading_text, update_info_panel_ant_count, update_info_panel_ant_hunger));

    }
}
