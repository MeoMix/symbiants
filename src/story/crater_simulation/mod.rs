mod background;
pub mod crater;

use bevy::prelude::*;

use crate::{
    app_state::{finalize_startup, AppState},
    settings::initialize_settings_resources,
};

use self::{
    background::{setup_background, teardown_background},
    crater::{
        setup_crater, setup_crater_ants, setup_crater_elements, setup_crater_grid, teardown_crater,
        Crater,
    },
};

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     OnEnter(AppState::CreateNewStory),
        //     ((
        //         (setup_crater, apply_deferred).chain(),
        //         (setup_crater_elements, apply_deferred).chain(),
        //         (setup_crater_ants, apply_deferred).chain(),
        //     )
        //         .chain()
        //         .before(finalize_startup)
        //         .after(setup_settings))
        //     .chain(),
        // );

        // app.add_systems(
        //     OnEnter(AppState::FinishSetup),
        //     (
        //         (setup_crater_grid, apply_deferred).chain(),
        //         (setup_background, apply_deferred).chain(),
        //     )
        //         .chain(),
        // );

        // app.add_systems(
        //     OnEnter(AppState::Cleanup),
        //     (teardown_background, teardown_crater),
        // );
    }
}