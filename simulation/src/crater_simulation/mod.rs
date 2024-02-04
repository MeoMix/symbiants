pub mod crater;

use self::{
    crater::register_crater,
    crater::{spawn_crater, Crater},
};
use super::{
    apply_deferred, despawn_model, insert_crater_grid, post_setup_clear_change_detection,
    settings::initialize_settings_resources, AppState, CleanupSet, FinishSetupSet,
};
use bevy::prelude::*;

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::BeginSetup), register_crater);

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (spawn_crater, apply_deferred)
                .chain()
                .after(initialize_settings_resources),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            ((insert_crater_grid, apply_deferred).chain(),)
                .chain()
                .before(post_setup_clear_change_detection)
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (despawn_model::<Crater>,).in_set(CleanupSet::SimulationCleanup),
        );
    }
}
