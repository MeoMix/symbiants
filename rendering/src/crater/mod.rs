pub mod background;

use bevy::prelude::*;
use simulation::{
    app_state::AppState, crater_simulation::crater::Crater, CleanupSet, FinishSetupSet,
};

use crate::common::{
    despawn_view,
    visible_grid::{VisibleGrid, VisibleGridState},
};

use self::background::{cleanup_background, spawn_background, CraterBackground};

pub struct CraterRenderingPlugin;

// TODO: Create a CraterGrid + CraterBackground
// Initially every spot in the grid is filled with air and the background draws a gray/brown floor
// Then need to spawn food at a location, put an ant sprite, and implement walking so that ant can grab it

impl Plugin for CraterRenderingPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     OnEnter(AppState::FinishSetup),
        //     (spawn_background)
        //         .chain()
        //         .in_set(FinishSetupSet::AfterSimulationFinishSetup),
        // );

        app.add_systems(
            OnEnter(VisibleGridState::Crater),
            (spawn_background, mark_crater_visible)
                .chain()
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnExit(VisibleGridState::Crater),
            (despawn_view::<CraterBackground>, mark_crater_hidden)
                .run_if(in_state(AppState::TellStory)),
        );

        app.add_systems(
            OnEnter(AppState::Cleanup),
            (despawn_view::<CraterBackground>, cleanup_background)
                .in_set(CleanupSet::BeforeSimulationCleanup),
        );
    }
}

pub fn mark_crater_visible(
    crater_query: Query<Entity, With<Crater>>,
    mut visible_grid: ResMut<VisibleGrid>,
) {
    visible_grid.0 = Some(crater_query.single());
}

pub fn mark_crater_hidden(mut visible_grid: ResMut<VisibleGrid>) {
    visible_grid.0 = None;
}
