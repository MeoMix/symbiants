mod background;
pub mod crater;

use bevy::prelude::*;

use crate::app_state::{finalize_startup, AppState};

use self::{
    background::{setup_background, teardown_background},
    crater::{setup_crater, setup_crater_elements, setup_crater_grid, teardown_crater, Crater},
};

use super::common::Zone;

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::CreateNewStory),
            ((
                (setup_crater, apply_deferred).chain(),
                (setup_crater_elements, apply_deferred).chain(),
            )
                .chain()
                .before(finalize_startup))
            .chain(),
        );

        app.add_systems(
            OnEnter(AppState::FinishSetup),
            (
                (ensure_crater_spatial_bundle, apply_deferred).chain(),
                (setup_crater_grid, apply_deferred).chain(),
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

// HACK: i'm reusing the same entity for view + model, but creating model first and reactively handling view props
// this results in warnings when I attach background as a child of crater because crater hasn't gained spatial bundle yet
// I would just spawn crater with it, but it's not persisted, so I need to insert it after loading Crater from storage
pub fn ensure_crater_spatial_bundle(
    crater_query: Query<Entity, (With<Zone>, With<Crater>)>,
    mut commands: Commands,
) {
    commands
        .entity(crater_query.single())
        .insert(SpatialBundle::default());
}
