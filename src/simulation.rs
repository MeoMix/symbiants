use bevy::prelude::*;

use crate::{
    ant::{move_ants_system, setup_ants},
    background::setup_background,
    elements::setup_elements,
    gravity::{
        ant_gravity_system, gravity_crush_system, gravity_stability_system, sand_gravity_system,
    },
    map::{periodic_save_world_state_system, setup_window_onunload_save_world_state},
    render::{render_carrying, render_rotation, render_scale, render_translation},
    time::{play_time_system, setup_fast_forward_time_system},
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems(
            (
                setup_fast_forward_time_system,
                setup_background,
                setup_elements,
                setup_ants,
                setup_window_onunload_save_world_state,
            )
                .chain(),
        );
        // TODO: need to be more aggressive in reusing state between update render and initial draw
        // currently if I turn off render systems there are minor graphical differences between initial draw and update render
        app.add_systems(
            (
                // move_ants runs first to avoid scenario where ant falls due to gravity and then moves in the same visual tick
                move_ants_system,
                // TODO: sand/ant gravity systems could run in parallel at the query level if effort is put into combining their logic.
                sand_gravity_system,
                gravity_crush_system,
                gravity_stability_system,
                ant_gravity_system,
                // Try to save world state periodically after updating world state.
                periodic_save_world_state_system,
                // Render world state after updating world state.
                // NOTE: all render methods can run in parallel but don't due to conflicting mutable Transform access
                render_translation,
                render_scale,
                render_rotation,
                render_carrying,
                play_time_system,
            )
                .chain()
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
