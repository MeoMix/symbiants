use bevy::prelude::*;
use bevy_save::{Snapshot, SnapshotSerializer, WorldSaveableExt};
use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use serde_json::Deserializer;
use std::{ops::Deref, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{settings::Settings, time::IsFastForwarding};

const LOCAL_STORAGE_KEY: &str = "world-save-state";

static SAVE_SNAPSHOT: Mutex<Option<String>> = Mutex::new(None);

// TODO: maybe try to save as DateTime<Utc> ?
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct LastSaveTime(pub i64);

// TODO: no way this should be here - it's like a separate module entirely?

// TODO: This runs awfully slow after switching to bevy_save away from manual Query reading
pub fn periodic_save_world_state(
    world: &mut World,
    mut last_snapshot_time: Local<f32>,
    mut last_save_time: Local<f32>,
) {
    let is_fast_forwarding = world.resource::<IsFastForwarding>();

    // Don't save while state is fast forwarding because it will cause a lot of lag.
    if is_fast_forwarding.0 {
        return;
    }

    let settings = world.resource::<Settings>();
    let time = world.resource::<Time>();

    if *last_snapshot_time != 0.0
        && time.raw_elapsed_seconds() - *last_snapshot_time
            < settings.auto_snapshot_interval_ms as f32 / 1000.0
    {
        return;
    }

    // Limit the lifetime of the lock so that `write_save_snapshot` is able to re-acquire
    {
        let mut save_snapshot = SAVE_SNAPSHOT.lock().unwrap();

        let mut writer: Vec<u8> = Vec::new();
        let mut serde = serde_json::Serializer::new(&mut writer);

        let snapshot = Snapshot::from_world_with_filter(world, |type_registration| {
            // Filter all view-related components from the snapshot. They'll get regenerated via ui systems' on_spawn_*
            !type_registration.type_name().starts_with("bevy")
        });

        let registry: &AppTypeRegistry = world.resource::<AppTypeRegistry>();
        let ser = SnapshotSerializer::new(&snapshot, registry);

        let result = ser.serialize(&mut serde);

        if result.is_ok() {
            *save_snapshot = Some(String::from_utf8(writer).unwrap());
            *last_snapshot_time = time.raw_elapsed_seconds();
        } else {
            error!("Failed to serialize snapshot: {:?}", result);
        }
    }

    if *last_save_time != 0.0
        && time.raw_elapsed_seconds() - *last_save_time
            < settings.auto_save_interval_ms as f32 / 1000.0
    {
        return;
    }

    if write_save_snapshot() {
        *last_save_time = time.raw_elapsed_seconds();

        // TODO: I think there's a bug here where I should be persisting LastSaveTime but I'm not? Need to double check
        // TODO: Rename LastSaveTime to something pertaining to the world time rather than impyling coupling to saving specifically.
        world.get_resource_mut::<LastSaveTime>().unwrap().0 = Utc::now().timestamp();
        info!("saved");
    }
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
