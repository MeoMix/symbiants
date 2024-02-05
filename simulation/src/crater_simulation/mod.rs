pub mod crater;

use self::{
    crater::register_crater,
    crater::{spawn_crater, spawn_crater_ants, spawn_crater_elements, Crater},
};
use super::{
    apply_deferred, despawn_model, insert_crater_grid, settings::initialize_settings_resources,
    AppState, CleanupSet, FinishSetupSet,
};
use bevy::prelude::*;

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::BeginSetup), register_crater);

        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            (
                // Call `apply_deferred` to ensure Settings (via `initialize_settings_resources`) is available for use.
                apply_deferred,
                spawn_crater,
                apply_deferred,
                (spawn_crater_elements, spawn_crater_ants),
            )
                .chain()
                .after(initialize_settings_resources),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (insert_crater_grid,)
                .chain()
                .in_set(FinishSetupSet::SimulationFinishSetup),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (despawn_model::<Crater>,).in_set(CleanupSet::SimulationCleanup),
        );
    }
}
