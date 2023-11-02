// TODO: probably merge crater and nest simulation and modularize at a lower level because there's a lot of core/common simulation code
use bevy::prelude::*;

// use crate::{story_state::StoryState, settings::Settings, nest::position::Position};

pub struct CraterSimulationPlugin;

impl Plugin for CraterSimulationPlugin {
    fn build(&self, _: &mut App) {
        info!("hello world");

        // Need to render a grid of squares representing the craters available area.
        // Start with square, can make it round after.

        // app.add_systems(OnEnter(StoryState::Initializing), ().chain());
    }
}