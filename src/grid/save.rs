use bevy::prelude::*;
use bevy_save::{Snapshot, SnapshotSerializer, WorldSaveableExt};
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use serde_json::{Deserializer, Serializer};
use std::{ops::Deref, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{settings::Settings, time::IsFastForwarding};

const LOCAL_STORAGE_KEY: &str = "world-save-state";

static SAVE_SNAPSHOT: Mutex<Option<String>> = Mutex::new(None);

// TODO: no way this should be here - it's like a separate module entirely?

// TODO: This runs awfully slow after switching to bevy_save away from manual Query reading
pub fn periodic_save_world_state(
    world: &mut World,
    mut last_snapshot_time: Local<f32>,
    mut last_save_time: Local<f32>,
) {
    // TODO: This is sort of an odd way to express this. I feel like maybe I want to be saving in "real-world time elapsed" not "in-game time elapsed"
    // Don't save while state is fast forwarding because it will cause a lot of lag.
    if world.resource::<IsFastForwarding>().0 {
        return;
    }

    if *last_snapshot_time != 0.0
        && world.resource::<Time>().raw_elapsed_seconds() - *last_snapshot_time
            < world.resource::<Settings>().auto_snapshot_interval_s as f32
    {
        return;
    }

    // Limit the lifetime of the lock so that `write_save_snapshot` is able to re-acquire
    {
        let snapshot = create_save_snapshot(world);

        if snapshot.is_some() {
            *SAVE_SNAPSHOT.lock().unwrap() = snapshot;
            *last_snapshot_time = world.resource::<Time>().raw_elapsed_seconds();
        }
    }

    if *last_save_time != 0.0
        && world.resource::<Time>().raw_elapsed_seconds() - *last_save_time < world.resource::<Settings>().auto_save_interval_s as f32
    {
        return;
    }

    if write_save_snapshot() {
        *last_save_time = world.resource::<Time>().raw_elapsed_seconds();

        info!("saved");
    }
}

fn create_save_snapshot(world: &mut World,) -> Option<String> {
    let mut writer: Vec<u8> = Vec::new();
    let mut serde = Serializer::new(&mut writer);

    let snapshot = Snapshot::from_world(world);
    let registry: &AppTypeRegistry = world.resource::<AppTypeRegistry>();

    let result = SnapshotSerializer::new(&snapshot, registry).serialize(&mut serde);

    if result.is_ok() {
        return Some(String::from_utf8(writer).unwrap());
    } else {
        error!("Failed to serialize snapshot: {:?}", result);
    }
    None
}

fn write_save_snapshot() -> bool {
    let save_snapshot = SAVE_SNAPSHOT.lock().unwrap();
    let save_result = LocalStorage::set(LOCAL_STORAGE_KEY, save_snapshot.deref().clone());

    if save_result.is_err() {
        error!(
            "Failed to save world state to local storage: {:?}",
            save_result
        );
    }

    save_result.is_ok()
}

pub fn setup_window_onunload_save_world_state() {
    let window = web_sys::window().expect("window not available");

    let on_beforeunload = Closure::wrap(Box::new(move |_| {
        write_save_snapshot();
    }) as Box<dyn FnMut(web_sys::BeforeUnloadEvent)>);

    let add_event_listener_result = window
        .add_event_listener_with_callback("beforeunload", on_beforeunload.as_ref().unchecked_ref());

    if add_event_listener_result.is_err() {
        error!(
            "Failed to add event listener for beforeunload: {:?}",
            add_event_listener_result
        );
    }

    on_beforeunload.forget();
}

pub fn load_existing_world(world: &mut World) -> bool {
    LocalStorage::get::<String>(LOCAL_STORAGE_KEY)
        .map_err(|e| {
            error!("Failed to load world state from local storage: {:?}", e);
        })
        .and_then(|saved_state| {
            let mut serde = Deserializer::from_str(&saved_state);
            world.deserialize(&mut serde).map_err(|e| {
                error!("Deserialization error: {:?}", e);
            })
        })
        .is_ok()
}
