mod background;
pub mod crater;

use bevy::prelude::*;

use crate::app_state::AppState;

use self::{
    background::{setup_background, teardown_background},
    crater::{setup_crater, teardown_crater},
};

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (setup_crater, apply_deferred).chain(),
                (setup_background, apply_deferred).chain(),
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (teardown_background, teardown_crater),
        );
    }
}
