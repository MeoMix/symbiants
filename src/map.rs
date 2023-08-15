use bevy::prelude::*;
use bevy_save::{Snapshot, SnapshotSerializer, WorldSaveableExt};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    ops::{Add, Deref, Mul},
    sync::Mutex,
};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    ant::{
        Angle, AntBundle, AntColor, AntInventory, AntName, AntOrientation, AntRole, Facing,
        Initiative,
    },
    element::{AirElementBundle, DirtElementBundle, Element},
    food::FoodCount,
    name_list::get_random_name,
    settings::Settings,
    time::IsFastForwarding,
    world_rng::Rng,
};

use chrono::{DateTime, Utc};

#[derive(
    Component, Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize, Reflect, Default,
)]
#[reflect(Component)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    #[allow(dead_code)]
    pub const ZERO: Self = Self::new(0, 0);
    pub const X: Self = Self::new(1, 0);
    pub const NEG_X: Self = Self::new(-1, 0);

    pub const Y: Self = Self::new(0, 1);
    pub const NEG_Y: Self = Self::new(0, -1);

    pub const ONE: Self = Self::new(1, 1);
    pub const NEG_ONE: Self = Self::new(-1, -1);

    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn as_world_position(&self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            // The view of the model position is just an inversion along the y-axis.
            y: -self.y as f32,
            z: 1.0,
        }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul for Position {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

#[derive(Resource, Debug)]
pub struct WorldMap {
    width: isize,
    height: isize,
    surface_level: isize,
    has_started_nest: bool,
    is_nested: bool,
    created_at: DateTime<Utc>,
    elements_cache: Vec<Vec<Entity>>,
}

pub const LOCAL_STORAGE_KEY: &str = "world-save-state";

pub fn setup_world_map(world: &mut World) {
    if !load_existing_world(world) {
        initialize_new_world(world);
    }

    let (width, height, surface_level) = {
        let settings = world.resource::<Settings>();
        (
            settings.world_width,
            settings.world_height,
            settings.get_surface_level(),
        )
    };

    let elements_cache = create_elements_cache(world, width as usize, height as usize);
    world.insert_resource(WorldMap::new(width, height, surface_level, elements_cache));
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

// Create a cache which allows for spatial querying of Elements. This is used to speed up
// most logic because there's a consistent need throughout the application to know what elements are
// at or near a given position.
pub fn create_elements_cache(world: &mut World, width: usize, height: usize) -> Vec<Vec<Entity>> {
    let mut elements_cache = vec![vec![Entity::PLACEHOLDER; width as usize]; height as usize];

    for (position, entity) in world
        .query_filtered::<(&mut Position, Entity), With<Element>>()
        .iter(&world)
    {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    elements_cache
}

pub fn initialize_new_world(world: &mut World) {
    let settings = Settings::default();

    world.insert_resource(settings);
    world.init_resource::<FoodCount>();
    world.init_resource::<LastSaveTime>();

    for x in 0..settings.world_height {
        for y in 0..settings.world_width {
            let position = Position::new(x, y);

            if y <= settings.get_surface_level() {
                world.spawn(AirElementBundle::new(position));
            } else {
                world.spawn(DirtElementBundle::new(position));
            }
        }
    }

    let ants = {
        let mut rng = world.resource_mut::<Rng>();

        let queen_ant = AntBundle::new(
            settings.get_random_surface_position(&mut rng),
            AntColor(settings.ant_color),
            AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
            AntInventory::default(),
            AntRole::Queen,
            AntName(String::from("Queen")),
            Initiative::new(&mut rng),
        );

        let worker_ants = (0..settings.initial_ant_worker_count)
            .map(|_| {
                AntBundle::new(
                    settings.get_random_surface_position(&mut rng),
                    AntColor(settings.ant_color),
                    AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
                    AntInventory::default(),
                    AntRole::Worker,
                    AntName(get_random_name(&mut rng)),
                    Initiative::new(&mut rng),
                )
            })
            .collect::<Vec<_>>();

        vec![queen_ant].into_iter().chain(worker_ants.into_iter())
    };

    world.spawn_batch(ants);
}

impl WorldMap {
    pub fn new(
        width: isize,
        height: isize,
        surface_level: isize,
        elements_cache: Vec<Vec<Entity>>,
    ) -> Self {
        WorldMap {
            width,
            height,
            surface_level,
            has_started_nest: false,
            is_nested: false,
            elements_cache,
            created_at: Utc::now(),
        }
    }

    pub fn width(&self) -> &isize {
        &self.width
    }

    pub fn height(&self) -> &isize {
        &self.height
    }

    pub fn surface_level(&self) -> &isize {
        &self.surface_level
    }

    pub fn has_started_nest(&self) -> &bool {
        &self.has_started_nest
    }

    pub fn start_nest(&mut self) {
        self.has_started_nest = true;
    }

    pub fn is_nested(&self) -> bool {
        self.is_nested
    }

    pub fn mark_nested(&mut self) {
        self.is_nested = true;
    }

    // round up so start at 1
    pub fn days_old(&self) -> i64 {
        let now = Utc::now();
        let duration = now - self.created_at;
        duration.num_days().add(1)
    }

    pub fn is_below_surface(&self, position: &Position) -> bool {
        position.y > self.surface_level
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    pub fn get_element(&self, position: Position) -> Option<&Entity> {
        self.elements_cache
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
    }

    pub fn get_element_expect(&self, position: Position) -> &Entity {
        self.get_element(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    pub fn set_element(&mut self, position: Position, entity: Entity) {
        self.elements_cache[position.y as usize][position.x as usize] = entity;
    }

    pub fn is_element(
        &self,
        elements_query: &Query<&Element>,
        position: Position,
        search_element: Element,
    ) -> bool {
        self.get_element(position).map_or(false, |&element| {
            elements_query
                .get(element)
                .map_or(false, |queried_element| *queried_element == search_element)
        })
    }

    // Returns true if every element in `positions` matches the provided Element type.
    // NOTE: This returns true if given 0 positions.
    pub fn is_all_element(
        &self,
        elements_query: &Query<&Element>,
        positions: &[Position],
        search_element: Element,
    ) -> bool {
        positions
            .iter()
            .all(|&position| self.is_element(elements_query, position, search_element))
    }
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

static SAVE_SNAPSHOT: Mutex<Option<String>> = Mutex::new(None);

// TODO: maybe try to save as DateTime<Utc> ?
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct LastSaveTime(pub i64);

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
