use bevy::prelude::*;

use crate::{
    ant::{move_ants_system, setup_ants},
    background::setup_background,
    elements::setup_elements,
    gravity::{ant_gravity_system, sand_gravity_system},
    map::save_world_state_system,
    render::{render_carrying, render_rotation, render_scale, render_translation},
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems((setup_ants, setup_elements, setup_background));

        app.add_systems(
            (
                // move_ants runs first to avoid scenario where ant falls due to gravity and then moves in the same visual tick
                move_ants_system,
                // TODO: sand/ant gravity systems could run in parallel at the query level if effort is put into combining their logic.
                sand_gravity_system,
                ant_gravity_system,
                // Try to save world state periodically after updating world state.
                save_world_state_system,
                // Render world state after updating world state.
                // NOTE: all render methods can run in parallel but don't due to conflicting mutable Transform access
                render_translation,
                render_scale,
                render_rotation,
                render_carrying,
            )
                .chain()
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
