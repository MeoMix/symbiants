use bevy::{ecs::query::WorldQuery, prelude::*};
use bevy_save::{
    Backend, DefaultDebugFormat, Error, Format, Pipeline, Snapshot, SnapshotBuilder,
    SnapshotSerializer, WorldSaveableExt,
};
use brotli::enc::BrotliEncoderInitParams;
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeSeed;
use serde::Serialize;
use std::{cell::RefCell, io::Read, io::Write, sync::Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::BeforeUnloadEvent;

use crate::{
    common::pheromone::Pheromone,
    crater_simulation::crater::Crater,
    nest_simulation::{ant::Ant, element::Element, nest::Nest},
    settings::Settings,
    story_time::{StoryRealWorldTime, StoryTime},
};

const LOCAL_STORAGE_KEY: &str = "world-save-state";
const LOAD_ERROR: &str = "Failed to load world state from local storage";
const DECOMPRESS_ERROR: &str = "Failed to decompress data";

static SAVE_SNAPSHOT: Mutex<Option<Vec<u8>>> = Mutex::new(None);

#[derive(WorldQuery)]
struct PersistentModelQueryFilter {
    _or: Or<(
        With<Ant>,
        With<Element>,
        With<Crater>,
        With<Nest>,
        With<Pheromone>,
    )>,
}

#[derive(Resource, Default)]
pub struct LastSnapshotTime(f32);

#[derive(Resource, Default)]
pub struct LastSaveTime(f32);

/// Provide an opportunity to write world state to disk.
/// This system does not run every time because saving is costly, but it does run periodically, rather than simply JIT,
/// to avoid losing too much state in the event of a crash.
/// NOTE: intentionally don't run immediately on first run because it's expensive and nothing has changed.
/// Let the full interval pass before creating anything rather than initializing on first run then waiting.
pub fn save(world: &mut World) {
    let current_time = world.resource::<Time<Real>>().elapsed_seconds();
    let last_snapshot_time = world.resource::<LastSnapshotTime>();
    let snapshot_interval = world.resource::<Settings>().snapshot_interval;
    if current_time - last_snapshot_time.0 < snapshot_interval as f32 {
        return;
    }

    if let Some(snapshot) = create_save_snapshot(world) {
        *SAVE_SNAPSHOT.lock().unwrap() = Some(snapshot);
        world.resource_mut::<LastSnapshotTime>().0 = current_time;
    } else {
        error!("Failed to create snapshot");
    }

    let save_interval = world.resource::<Settings>().save_interval;
    let last_save_time = world.resource::<LastSaveTime>();
    if current_time - last_save_time.0 < save_interval as f32 {
        return;
    }

    if write_save_snapshot() {
        world.resource_mut::<LastSaveTime>().0 = current_time;
    }
}

fn create_save_snapshot(world: &mut World) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut serde = rmp_serde::Serializer::new(&mut buffer);

    // Persistent entities must have an Id marker because Id is fit for uniquely identifying across sessions.
    // NOTE: Technically this could also include InventoryItem, but Element matches it (just by chance for now though?)
    let mut model_query = world.query_filtered::<Entity, PersistentModelQueryFilter>();

    model_query.update_archetypes(world);
    let readonly_model_query = model_query.as_readonly();
    let snapshot = build_snapshot(world, readonly_model_query);

    let registry: &AppTypeRegistry = world.resource::<AppTypeRegistry>();
    let result = SnapshotSerializer::new(&snapshot, registry).serialize(&mut serde);

    if result.is_ok() {
        return Some(buffer);
    } else {
        error!("Failed to serialize snapshot: {:?}", result);
    }

    None
}

fn write_save_snapshot() -> bool {
    let save_snapshot = SAVE_SNAPSHOT.lock().unwrap();

    let buffer = match save_snapshot.as_ref() {
        Some(buffer) => buffer,
        // SAVE_SNAPSHOT can be empty during the first few seconds of app load because snapshots are taken periodically.
        None => return false,
    };

    // Compress snapshot using Brotli. In testing, this reduces a 4mb save file to 0.5mb with compression quality: 1.
    let mut params = BrotliEncoderInitParams();
    params.quality = 1; // Max compression (0-11 range)

    let mut compressed_data = brotli::CompressorWriter::with_params(Vec::new(), 4096, &params);
    compressed_data
        .write_all(buffer)
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

pub fn bind_save_onbeforeunload() {
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

pub fn unbind_save_onbeforeunload() {
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

pub fn delete_save_file() {
    LocalStorage::delete(LOCAL_STORAGE_KEY);
}

pub fn initialize_save_resources(mut commands: Commands) {
    commands.init_resource::<CompressedWebStorageBackend>();
    commands.init_resource::<LastSnapshotTime>();
    commands.init_resource::<LastSaveTime>();
}

pub fn remove_save_resources(mut commands: Commands) {
    commands.remove_resource::<CompressedWebStorageBackend>();
    commands.remove_resource::<LastSnapshotTime>();
    commands.remove_resource::<LastSaveTime>();
}

pub fn load(world: &mut World) -> bool {
    let mut model_query = world.query_filtered::<Entity, PersistentModelQueryFilter>();
    model_query.update_archetypes(world);

    let readonly_model_query = model_query.as_readonly();

    world
        .load(SaveLoadPipeline::new(readonly_model_query))
        .is_ok()
}

struct SaveLoadPipeline<'q> {
    key: String,
    readonly_model_query: &'q QueryState<Entity, PersistentModelQueryFilter>,
}

impl<'q> SaveLoadPipeline<'q> {
    pub fn new(readonly_model_query: &'q QueryState<Entity, PersistentModelQueryFilter>) -> Self {
        Self {
            key: LOCAL_STORAGE_KEY.to_string(),
            readonly_model_query,
        }
    }
}

impl<'q> Pipeline for SaveLoadPipeline<'q> {
    type Backend = CompressedWebStorageBackend;
    type Format = DefaultDebugFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.key
    }

    fn capture_seed(&self, builder: SnapshotBuilder) -> Snapshot {
        build_snapshot(builder.world(), self.readonly_model_query)
    }

    fn apply_seed(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), bevy_save::Error> {
        snapshot.applier(world).apply()
    }
}

#[derive(Default, Resource)]
pub struct CompressedWebStorageBackend;

impl<'a> Backend<&'a str> for CompressedWebStorageBackend {
    fn save<F: Format, T: Serialize>(&self, _key: &str, _value: &T) -> Result<(), Error> {
        Err(Error::custom(
            "Not implemented - expected to save by writing snapshot manually for now",
        ))
    }

    fn load<F: Format, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        &self,
        key: &str,
        seed: S,
    ) -> Result<T, Error> {
        // Attempt to retrieve the compressed state from local storage
        let compressed_saved_state = LocalStorage::get::<Vec<u8>>(key).map_err(|e| {
            error!("{}: {:?}", LOAD_ERROR, e);
            Error::custom(LOAD_ERROR)
        })?;

        // Initialize the decompressor
        let mut decompressor = brotli::Decompressor::new(&compressed_saved_state[..], 4096);
        let mut decompressed_data = Vec::new();

        // Attempt to decompress the data
        decompressor
            .read_to_end(&mut decompressed_data)
            .map_err(|e| {
                error!("{}: {:?}", DECOMPRESS_ERROR, e);
                Error::custom(DECOMPRESS_ERROR)
            })?;

        // Deserialize the data
        let mut deserializer = rmp_serde::Deserializer::new(&decompressed_data[..]);
        seed.deserialize(&mut deserializer).map_err(Error::loading)
    }
}

fn build_snapshot(
    world: &World,
    readonly_model_query: &QueryState<Entity, PersistentModelQueryFilter>,
) -> Snapshot {
    Snapshot::builder(world)
        .extract_entities(readonly_model_query.iter_manual(world))
        .extract_resource::<Settings>()
        .extract_resource::<StoryTime>()
        .extract_resource::<StoryRealWorldTime>()
        .build()
}
