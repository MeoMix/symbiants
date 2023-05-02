use bevy::prelude::*;

use crate::{
    ant::{move_ant, setup_ants},
    background::setup_background,
    elements::setup_elements,
    gravity::{ant_gravity_system, sand_gravity_system},
    render::{render_carrying, render_rotation, render_scale, render_translation},
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_systems((setup_ants, setup_elements, setup_background));

        app.add_systems(
            (
                // NOTE: move_ant needs to run first to avoid situation where ant falls + moves in same frame
                move_ant,
                // TODO: sand/ant gravity systems can run in parallel, but not clear how to express that without allowing render to run
                sand_gravity_system.after(move_ant),
                ant_gravity_system.after(sand_gravity_system),
                // Rendering systems need to run after all other systems, but can run in any order.
                render_translation
                    .after(ant_gravity_system)
                    .ambiguous_with(render_scale)
                    .ambiguous_with(render_rotation)
                    .ambiguous_with(render_carrying),
                render_scale
                    .after(ant_gravity_system)
                    .ambiguous_with(render_translation)
                    .ambiguous_with(render_rotation)
                    .ambiguous_with(render_carrying),
                render_rotation
                    .after(ant_gravity_system)
                    .ambiguous_with(render_translation)
                    .ambiguous_with(render_scale)
                    .ambiguous_with(render_carrying),
                render_carrying
                    .after(ant_gravity_system)
                    .ambiguous_with(render_translation)
                    .ambiguous_with(render_scale)
                    .ambiguous_with(render_rotation),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
