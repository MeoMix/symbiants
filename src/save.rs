use bevy::prelude::*;
use bevy_save::{Snapshot, SnapshotSerializer, WorldSaveableExt};
use brotli::{enc::BrotliEncoderInitParams, CompressorWriter, Decompressor};
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use serde_json::{Deserializer, Serializer};
use std::{cell::RefCell, io::Read, io::Write, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::BeforeUnloadEvent;

use crate::settings::Settings;

const LOCAL_STORAGE_KEY: &str = "world-save-state";

static SAVE_SNAPSHOT: Mutex<Option<String>> = Mutex::new(None);

// TODO: Support saving on non-WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn save() {}

/// Provide an opportunity to write world state to disk.
/// This system does not run every time because saving is costly, but it does run periodically, rather than simply JIT,
/// to avoid losing too much state in the event of a crash.
#[cfg(target_arch = "wasm32")]
pub fn save(world: &mut World, mut last_snapshot_time: Local<f32>, mut last_save_time: Local<f32>) {
    let current_time = world.resource::<Time>().raw_elapsed_seconds();
    let snapshot_interval = world.resource::<Settings>().snapshot_interval;
    if *last_snapshot_time != 0.0 && current_time - *last_snapshot_time < snapshot_interval as f32 {
        return;
    }

    if let Some(snapshot) = create_save_snapshot(world) {
        *SAVE_SNAPSHOT.lock().unwrap() = Some(snapshot);
        *last_snapshot_time = current_time;
    } else {
        error!("Failed to create snapshot");
    }

    let save_interval = world.resource::<Settings>().save_interval;
    if *last_save_time != 0.0 && current_time - *last_save_time < save_interval as f32 {
        return;
    }

    if write_save_snapshot() {
        *last_save_time = current_time;
    }
}

fn create_save_snapshot(world: &mut World) -> Option<String> {
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

    // Compress snapshot using Brotli. In testing, this reduces a 4mb save file to 0.5mb with compression quality: 1.
    let mut params = BrotliEncoderInitParams();
    params.quality = 1; // Max compression (0-11 range)

    let mut compressed_data = CompressorWriter::with_params(Vec::new(), 4096, &params);
    compressed_data
        .write_all(&save_snapshot.as_ref().unwrap().as_bytes())
        .expect("Failed to write to compressor");

    let save_result = LocalStorage::set(LOCAL_STORAGE_KEY, compressed_data.into_inner());

    if save_result.is_err() {
        error!(
            "Failed to save world state to local storage: {:?}",
            save_result
        );
    }

    save_result.is_ok()
}

thread_local! {
    static ON_BEFORE_UNLOAD: RefCell<Option<Closure<dyn FnMut(BeforeUnloadEvent) -> bool>>> = RefCell::new(None);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_save() {}

#[cfg(target_arch = "wasm32")]
pub fn setup_save() {
    let window = web_sys::window().expect("window not available");

    ON_BEFORE_UNLOAD.with(|opt_closure| {
        let closure = Closure::wrap(Box::new(move |_| {
            write_save_snapshot();
            // Tell browser not to interrupt the unload
            false
        }) as Box<dyn FnMut(BeforeUnloadEvent) -> bool>);

        window
            .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())
            .expect("Failed to add event listener for beforeunload");

        *opt_closure.borrow_mut() = Some(closure);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn teardown_save() {}

#[cfg(target_arch = "wasm32")]
pub fn teardown_save() {
    LocalStorage::delete(LOCAL_STORAGE_KEY);

    let window = web_sys::window().expect("window not available");

    ON_BEFORE_UNLOAD.with(|opt_closure| {
        if let Some(on_beforeunload) = opt_closure.borrow_mut().take() {
            window
                .remove_event_listener_with_callback(
                    "beforeunload",
                    on_beforeunload.as_ref().unchecked_ref(),
                )
                .unwrap();
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load(world: &mut World) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
pub fn load(world: &mut World) -> bool {
    LocalStorage::get::<Vec<u8>>(LOCAL_STORAGE_KEY)
        .map_err(|e| {
            error!("Failed to load world state from local storage: {:?}", e);
        })
        .and_then(|compressed_saved_state| {
            let mut decompressor = Decompressor::new(&compressed_saved_state[..], 4096);
            let mut decompressed_data = Vec::new();

            if let Err(e) = decompressor.read_to_end(&mut decompressed_data) {
                error!("Failed to decompress data: {:?}", e);
            }

            let saved_state = String::from_utf8(decompressed_data).map_err(|e| {
                error!("Failed to convert decompressed bytes to string: {:?}", e);
            })?;

            let mut serde = Deserializer::from_str(&saved_state);
            world.deserialize(&mut serde).map_err(|e| {
                error!("Deserialization error: {:?}", e);
            })
        })
        .is_ok()
}
