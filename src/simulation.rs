use bevy::prelude::*;

use crate::{
    ant::{ants_hunger_system, move_ants_system, setup_ants, ants_birthing_system, on_spawn_ant},
    background::setup_background,
    elements::setup_elements,
    gravity::{
        ant_gravity_system, element_gravity_system, gravity_crush_system, gravity_stability_system,
    },
    map::{periodic_save_world_state_system, setup_window_onunload_save_world_state},
    mouse::{handle_mouse_clicks, is_pointer_captured_system, IsPointerCaptured},
    render::{render_carrying, render_orientation, render_translation},
    time::{play_time_system, setup_fast_forward_time_system},
    food::FoodCount,
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodCount>();
        app.insert_resource(IsPointerCaptured(false));

        app.add_systems(Startup,
            (
                setup_fast_forward_time_system,
                setup_background,
                setup_elements,
                setup_ants,
                setup_window_onunload_save_world_state,
            )
                .chain(),
        );

        app.add_systems(Update, (is_pointer_captured_system, handle_mouse_clicks).chain());

        app.add_systems(FixedUpdate, 
            (
                // move_ants runs first to avoid scenario where ant falls due to gravity and then moves in the same visual tick
                move_ants_system,
                ants_hunger_system,
                ants_birthing_system,
                element_gravity_system,
                gravity_crush_system,
                gravity_stability_system,
                ant_gravity_system,
                // Try to save world state periodically after updating world state.
                periodic_save_world_state_system,
                // Render world state after updating world state.
                // NOTE: all render methods can run in parallel but don't due to conflicting mutable Transform access
                render_translation,
                render_orientation,
                render_carrying,
                play_time_system,
            )
                .chain(),
        );

        app.add_systems(PostUpdate, on_spawn_ant);
    }
}
