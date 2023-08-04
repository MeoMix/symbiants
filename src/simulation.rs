use bevy::prelude::*;

use crate::{
    ant::{hunger::ants_hunger, move_ants, setup_ants, birthing::ants_birthing, ui::on_spawn_ant},
    background::setup_background,
    element::{setup_elements, ui::on_spawn_element},
    gravity::{
        ant_gravity, element_gravity, gravity_crush, gravity_stability,
    },
    map::{periodic_save_world_state, setup_window_onunload_save_world_state},
    mouse::{handle_mouse_clicks, is_pointer_captured, IsPointerCaptured},
    render::{render_carrying, render_orientation, render_translation},
    time::{play_time, setup_fast_forward_time},
    food::FoodCount,
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodCount>();
        app.insert_resource(IsPointerCaptured(false));

        app.add_systems(Startup,
            (
                setup_fast_forward_time,
                setup_background,
                setup_elements,
                setup_ants,
                setup_window_onunload_save_world_state,
            )
                .chain(),
        );

        // NOTE: don't process user input events in FixedUpdate because events in FixedUpdate are broken
        app.add_systems(Update, (is_pointer_captured, handle_mouse_clicks).chain());

        app.add_systems(FixedUpdate, 
            (
                // move_ants runs first to avoid scenario where ant falls due to gravity and then moves in the same visual tick
                move_ants,
                ants_hunger,
                ants_birthing,
                element_gravity,
                gravity_crush,
                gravity_stability,
                ant_gravity,
                // Try to save world state periodically after updating world state.
                periodic_save_world_state,
                // Render world state after updating world state.
                // NOTE: all render methods can run in parallel but don't due to conflicting mutable Transform access
                (render_translation, render_orientation, render_carrying).chain(),
                // NOTE: apply deferred fixes a bug where ant drops dirt infront of itself, then digs that dirt immediately and despawns it,
                // and this races the on_spawn_element logic which wanted to draw the dropped dirt.
                apply_deferred,
                // IMPORTANT: `on_spawn` systems must run at the end of `FixedUpdate` because `PostUpdate` does not guaranteee 1:1 runs with `FixedUpdate`
                // As such, while it's OK for rendering to become de-synced, it's not OK for underlying caches to become de-synced.
                (on_spawn_ant, on_spawn_element).chain(),
                play_time,
            )
                .chain(),
        );

        // NOTE: maybe turn this on if need to handle user input events?
        // app.add_systems(PostUpdate, (on_spawn_ant, on_spawn_element));
    }
}
