use crate::common::{LoadProgress, SimulationLoadProgress};
use bevy::prelude::*;

pub fn save() {}

pub fn bind_save_onbeforeunload() {}

pub fn unbind_save_onbeforeunload() {}

pub fn delete_save_file() {}

pub fn load_save_file(world: &mut World) {
    // TODO: Support saving on non-WASM targets.
    world.resource_mut::<SimulationLoadProgress>().save_file = LoadProgress::Failure;
}

pub fn initialize_save_resources() {}

pub fn remove_save_resources() {}
