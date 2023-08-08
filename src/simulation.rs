use bevy::prelude::*;

use crate::{
    ant::{
        act::ants_act,
        ants_initiative,
        birthing::ants_birthing,
        hunger::ants_hunger,
        setup_ants,
        ui::{on_spawn_ant, on_update_ant_inventory, on_update_ant_orientation, on_update_ant_dead},
        walk::ants_walk,
    },
    background::setup_background,
    common::ui::on_update_position,
    element::{setup_elements, ui::on_spawn_element},
    food::FoodCount,
    gravity::{ant_gravity, element_gravity, gravity_crush, gravity_stability},
    map::{periodic_save_world_state, setup_window_onunload_save_world_state, WorldMap},
    mouse::{handle_mouse_clicks, is_pointer_captured, IsPointerCaptured},
    settings::Settings,
    time::{play_time, setup_fast_forward_time, IsFastForwarding, PendingTicks, DEFAULT_TICK_RATE},
    world_rng::WorldRng,
};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoodCount>()
            .init_resource::<IsPointerCaptured>()
            .init_resource::<Settings>()
            .init_resource::<WorldRng>()
            .init_resource::<WorldMap>()
            .init_resource::<IsFastForwarding>()
            .init_resource::<PendingTicks>();

        // TODO: Move this into time plugin? idk
        // Defines the amount of time that should elapse between each physics step.
        // app.insert_resource(FixedTime::new_from_secs(0.2 / 60.0));
        app.insert_resource(FixedTime::new_from_secs(DEFAULT_TICK_RATE));

        app.add_systems(
            Startup,
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

        app.add_systems(
            FixedUpdate,
            (
                // TODO: revisit this idea - I want all simulation systems to be able to run in parallel.
                (
                    // Ants move before acting because positions update instantly, but actions use commands to mutate the world and are deferred + batched.
                    // By applying movement first, commands do not need to anticipate ants having moved, but the opposite would not be true.
                    ants_walk,
                    // Apply specific ant actions in priority order because ants take a maximum of one action per tick.
                    // An ant should not starve to hunger due to continually choosing to dig a tunnel, etc.
                    ants_hunger,
                    ants_birthing,
                    ants_act,
                    // Reset initiative only after all actions have occurred to ensure initiative properly throttles actions-per-tick.
                    ants_initiative,
                    // TODO: currently, element_gravity intentionally runs after ants_walk to ensure that if an ant is falling that it doesn't move
                    // the same tick it falls, but this should be enforced via a component.
                    element_gravity,
                    gravity_crush,
                    gravity_stability,
                    ant_gravity,
                )
                    .chain(),
                // Bevy doesn't have support for PreUpdate/PostUpdate lifecycle from within FixedUpdate.
                // In an attempt to simulate this behavior, manually call `apply_deferred` because this would occur
                // when moving out of the Update stage and into the PostUpdate stage.
                // This is an important action which prevents panics while maintaining simpler code.
                // Without this, an Element might be spawned, and then despawned, with its initial render command still enqueued.
                // This would result in a panic due to missing Element entity unless the render command was rewritten manually
                // to safeguard against missing entity at time of command application.
                apply_deferred,
                // Provide an opportunity to write world state to disk.
                // This system does not run every time because saving is costly, but it does run periodically, rather than simply JIT,
                // to avoid losing too much state in the event of a crash.
                periodic_save_world_state,
                // Ensure render state reflects simulation state after having applied movements and command updates.
                (
                    on_update_position,
                    on_update_ant_orientation,
                    on_update_ant_inventory,
                    on_update_ant_dead,
                )
                    .chain(),
                (on_spawn_ant, on_spawn_element).chain(),
                play_time,
            )
                .chain(),
        );

        // NOTE: maybe turn this on if need to handle user input events?
        // app.add_systems(PostUpdate, (on_spawn_ant, on_spawn_element));
    }
}
