use bevy::prelude::*;
use bevy_save::{Snapshot, SnapshotSerializer, WorldSaveableExt};
use gloo_storage::{LocalStorage, Storage};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Deref, Mul},
    sync::Mutex,
};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    ant::{Angle, AntBundle, AntInventory, AntOrientation, AntRole, Facing},
    element::{AirElementBundle, DirtElementBundle, Element},
    food::FoodCount,
    name_list::NAMES,
    settings::Settings,
    time::IsFastForwarding,
    world_rng::WorldRng,
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
    elements_cache: Option<Vec<Vec<Entity>>>,
}

pub const LOCAL_STORAGE_KEY: &str = "world-save-state";

pub fn setup_load_state(world: &mut World) {
    // Deserialize world state from local storage if possible otherwise initialize the world from scratch
    if let Ok(saved_state) = LocalStorage::get::<String>(LOCAL_STORAGE_KEY) {
        let mut serde = serde_json::Deserializer::from_str(&saved_state);

        let deserialization_result = world.deserialize(&mut serde);

        if deserialization_result.is_ok() {
            let settings = world.resource::<Settings>();
            let surface_level = (settings.world_height as f32
                - (settings.world_height as f32 * settings.initial_dirt_percent))
                as isize;

            let mut world_map =
                WorldMap::new(settings.world_width, settings.world_height, surface_level);

            let mut elements = world.query_filtered::<(&mut Position, Entity), With<Element>>();

            for (position, entity) in elements.iter(&world) {
                world_map.set_element(*position, entity);
            }

            world.insert_resource(world_map);

            return;
        } else {
            error!("Result: {:?}", deserialization_result);
        }
    }

    let settings = Settings::default();

    let surface_level = (settings.world_height as f32
        - (settings.world_height as f32 * settings.initial_dirt_percent))
        as isize;

    world.insert_resource(WorldMap::new(
        settings.world_width,
        settings.world_height,
        surface_level,
    ));

    for row_index in 0..settings.world_height {
        for column_index in 0..settings.world_width {
            let position = Position {
                x: column_index,
                y: row_index,
            };

            // TODO: spawn_batch???
            let entity = if row_index <= surface_level {
                world.spawn(AirElementBundle::new(position)).id()
            } else {
                world.spawn(DirtElementBundle::new(position)).id()
            };

            world
                .resource_mut::<WorldMap>()
                .set_element(position, entity);
        }
    }

    let ants = {
        let mut world_rng = world.get_resource_mut::<WorldRng>().unwrap();
        let queen_ant = create_queen_ant(&mut world_rng, surface_level, &settings);
        let worker_ants = create_worker_ants(&mut world_rng, surface_level, &settings);
        vec![queen_ant].into_iter().chain(worker_ants.into_iter())
    };

    for ant in ants {
        world.spawn(ant);
    }

    world.init_resource::<Settings>();
    world.init_resource::<FoodCount>();
    world.init_resource::<LastSaveTime>();
}

fn create_queen_ant(
    world_rng: &mut WorldRng,
    surface_level: isize,
    settings: &Settings,
) -> AntBundle {
    let x = world_rng.0.gen_range(0..1000) % settings.world_width;
    let y = surface_level;
    let facing = if world_rng.0.gen_bool(0.5) {
        Facing::Left
    } else {
        Facing::Right
    };
    AntBundle::new(
        Position::new(x, y),
        settings.ant_color,
        AntOrientation::new(facing, Angle::Zero),
        AntInventory(None),
        AntRole::Queen,
        "Queen",
        &mut world_rng.0,
    )
}

fn create_worker_ants(
    world_rng: &mut WorldRng,
    surface_level: isize,
    settings: &Settings,
) -> Vec<AntBundle> {
    (0..settings.initial_ant_worker_count)
        .map(|_| {
            let x = world_rng.0.gen_range(0..1000) % settings.world_width;
            let y = surface_level;
            let facing = if world_rng.0.gen_bool(0.5) {
                Facing::Left
            } else {
                Facing::Right
            };
            let name: &str = NAMES[world_rng.0.gen_range(0..NAMES.len())].clone();
            AntBundle::new(
                Position::new(x, y),
                settings.ant_color,
                AntOrientation::new(facing, Angle::Zero),
                AntInventory(None),
                AntRole::Worker,
                name,
                &mut world_rng.0,
            )
        })
        .collect()
}

impl WorldMap {
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

    pub fn new(width: isize, height: isize, surface_level: isize) -> Self {
        WorldMap {
            width,
            height,
            surface_level,
            has_started_nest: false,
            is_nested: false,
            elements_cache: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    pub fn get_element(&self, position: Position) -> Option<&Entity> {
        self.elements_cache
            .as_ref()?
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
    }

    pub fn get_element_expect(&self, position: Position) -> &Entity {
        self.get_element(position).expect(&format!(
            "Element entity not found at the position: {:?}",
            position
        ))
    }

    // NOTE: although this logic supports expanding the 2D vector - this should only occur during initialization
    // Afterward, vector should always be the same size as the world. Decided resizing vector was better than implying entries
    // in the vector might be None while maintaing a fixed length vector.
    pub fn set_element(&mut self, position: Position, entity: Entity) {
        if self.elements_cache.is_none() {
            self.elements_cache = Some(vec![
                vec![entity.clone(); position.x as usize + 1];
                position.y as usize + 1
            ]);
        }

        let cache = self.elements_cache.as_mut().unwrap();
        if position.y as usize >= cache.len() {
            cache.resize(
                position.y as usize + 1,
                vec![entity.clone(); position.x as usize + 1],
            );
        }

        let row = &mut cache[position.y as usize];
        if position.x as usize >= row.len() {
            row.resize(position.x as usize + 1, entity.clone());
        }

        row[position.x as usize] = entity;
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
