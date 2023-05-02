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
                // TODO: I think these can run in any order?
                sand_gravity_system.after(move_ant),
                ant_gravity_system.after(sand_gravity_system),
                // TODO: These can run in any order - how to express that?
                render_translation.after(ant_gravity_system),
                render_scale.after(render_translation),
                render_rotation.after(render_scale),
                render_carrying.after(render_rotation),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}
