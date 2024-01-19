// TODO: Support saving on non-WASM targets.
use bevy::prelude::*;

pub fn save() {}

pub fn bind_save_onbeforeunload() {}

pub fn unbind_save_onbeforeunload() {}

pub fn delete_save_file() {}

pub fn load(_world: &mut World) -> bool {
    false
}

pub fn initialize_save_resources() {}
