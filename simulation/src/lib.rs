pub mod app_state;
pub mod common;
pub mod crater_simulation;
pub mod external_event;
pub mod nest_simulation;
pub mod save;
pub mod settings;
pub mod simulation_timestep;
pub mod story_time;

use self::{
    app_state::AppState, common::despawn_model,
    simulation_timestep::run_simulation_update_schedule, story_time::StoryPlaybackState,
};
use bevy::{
    app::{MainScheduleOrder, RunFixedUpdateLoop},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use bevy_save::SavePlugin;
use common::CommonSimulationPlugin;
use crater_simulation::{crater::insert_crater_grid, CraterSimulationPlugin};
use nest_simulation::NestSimulationPlugin;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct RunSimulationUpdateLoop;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Clone, Hash)]
pub struct SimulationUpdate;

// TODO: I'm not absolutely convinced these are good practice. It feels like this is competing with AppState transition.
// An alternative would be to have an AppState for "SimulationFinishSetup" and "RenderingFinishSetup"
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Hash)]
pub enum FinishSetupSet {
    BeforeSimulationFinishSetup,
    SimulationFinishSetup,
    AfterSimulationFinishSetup,
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Hash)]
pub enum CleanupSet {
    BeforeSimulationCleanup,
    SimulationCleanup,
    AfterSimulationCleanup,
}

/// First and Last run even in the simulation is paused.
/// This is useful for having the simulation react to user input when paused.
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Hash)]
pub enum SimulationTickSet {
    First,
    PreSimulationTick,
    SimulationTick,
    PostSimulationTick,
    Last,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // Only want SavePlugin not SavePlugins - just need basic snapshot logic not UI persistence or save/load methods.
        app.add_plugins(SavePlugin);
        app.add_systems(PreStartup, insert_simulation_schedule);
        app.init_schedule(RunSimulationUpdateLoop);
        app.add_systems(
            RunSimulationUpdateLoop,
            run_simulation_update_schedule.run_if(in_state(AppState::TellStory)),
        );

        app.add_state::<StoryPlaybackState>();
        // TODO: AppState feels weird to live in Simulation
        app.add_state::<AppState>();

        app.configure_sets(
            OnEnter(AppState::FinishSetup),
            (
                FinishSetupSet::BeforeSimulationFinishSetup,
                FinishSetupSet::SimulationFinishSetup,
                FinishSetupSet::AfterSimulationFinishSetup,
            )
                .chain(),
        );

        app.configure_sets(
            SimulationUpdate,
            (
                SimulationTickSet::First,
                SimulationTickSet::PreSimulationTick,
                SimulationTickSet::SimulationTick,
                SimulationTickSet::PostSimulationTick,
                SimulationTickSet::Last,
            )
                .chain(),
        );

        app.configure_sets(
            OnEnter(AppState::Cleanup),
            (
                CleanupSet::BeforeSimulationCleanup,
                CleanupSet::SimulationCleanup,
                CleanupSet::AfterSimulationCleanup,
            )
                .chain(),
        );

        app.add_plugins((
            CommonSimulationPlugin,
            NestSimulationPlugin,
            CraterSimulationPlugin,
        ));
    }
}

pub fn insert_simulation_schedule(mut main_schedule_order: ResMut<MainScheduleOrder>) {
    main_schedule_order.insert_after(RunFixedUpdateLoop, RunSimulationUpdateLoop);
}
