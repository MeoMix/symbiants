use bevy::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Deref, Mul},
    sync::Mutex,
};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    ant::{Angle, AntBehavior, AntColor, AntName, AntOrientation, AntSaveState, AntTimer, Facing},
    elements::{Element, ElementSaveState},
    name_list::NAMES,
    settings::Settings,
    time::IsFastForwarding,
    world_rng::WorldRng,
};

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};

#[derive(Component, Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
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

// TODO: This should probably persist the settings it was generated with to prevent desync
// TODO: *no* idea if this is an acceptable way to persist state. It seems very OOP-y, but
// Bevy scenes did not seem like the right tool for the job, either.
#[derive(Default, Debug, Serialize, Deserialize, Resource)]
pub struct WorldSaveState {
    #[serde(with = "ts_seconds")]
    pub time_stamp: DateTime<Utc>,
    pub elements: Vec<ElementSaveState>,
    pub ants: Vec<AntSaveState>,
}

#[derive(Resource)]
pub struct WorldMap {
    // TODO: consider making width/height here `usize` since never expect them to be negative
    width: isize,
    height: isize,
    surface_level: isize,
    // TODO: Should not have this be public
    pub initial_state: WorldSaveState,
    // TODO: refactor code such that Option isn't needed here - always instantiate with properly populated elements
    pub elements: Vec<Vec<Option<Entity>>>,
}

pub const LOCAL_STORAGE_KEY: &str = "world-save-state";

impl FromWorld for WorldMap {
    fn from_world(world: &mut World) -> Self {
        // TODO: this feels like a hack?
        let mut settings = Settings::default();
        world.resource_scope(|_, settings_mut: Mut<Settings>| {
            settings = settings_mut.clone();
        });

        let surface_level = (settings.world_height as f32
            - (settings.world_height as f32 * settings.initial_dirt_percent))
            as isize;

        if let Ok(saved_state) = LocalStorage::get::<WorldSaveState>(LOCAL_STORAGE_KEY) {
            return WorldMap::new(
                settings.world_width,
                settings.world_height,
                surface_level,
                saved_state,
                None,
            );
        }

        let air = (0..(surface_level + 1)).flat_map(|row_index| {
            (0..settings.world_width).map(move |column_index| ElementSaveState {
                element: Element::Air,
                position: Position {
                    x: column_index,
                    y: row_index,
                },
            })
        });

        let dirt = ((surface_level + 1)..settings.world_height).flat_map(|row_index| {
            (0..settings.world_width).map(move |column_index| ElementSaveState {
                element: Element::Dirt,
                position: Position {
                    x: column_index,
                    y: row_index,
                },
            })
        });

        let mut world_rng = world.get_resource_mut::<WorldRng>().unwrap();
        let ants = (0..settings.initial_ant_count).map(|_| {
            // Put the ant at a random location along the x-axis that fits within the bounds of the world.
            let x = world_rng.0.gen_range(0..1000) % settings.world_width;
            // Put the ant on the dirt.
            let y = surface_level;

            // Randomly position ant facing left or right.
            let facing = if world_rng.0.gen_bool(0.5) {
                Facing::Left
            } else {
                Facing::Right
            };

            let name: &str = NAMES[world_rng.0.gen_range(0..NAMES.len())].clone();

            AntSaveState {
                position: Position::new(x, y),
                color: AntColor(settings.ant_color),
                orientation: AntOrientation::new(facing, Angle::Zero),
                behavior: AntBehavior::Wandering,
                timer: AntTimer::new(&AntBehavior::Wandering, &mut world_rng.0),
                name: AntName(name.to_string()),
            }
        });

        // let ants = [
        //     AntSaveState {
        //         position: Position::new(5, 5),
        //         color: settings.ant_color,
        //         facing: Facing::Left,
        //         angle: Angle::Zero,
        //         behavior: AntBehavior::Carrying,
        //         name: "ant1".to_string(),
        //     },
        //     Ant::new(
        //         Position::new(10, 5),
        //         settings.ant_color,
        //         Facing::Left,
        //         Angle::Ninety,
        //         AntBehavior::Carrying,
        //         "ant2".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(15, 5),
        //         settings.ant_color,
        //         Facing::Left,
        //         Angle::OneHundredEighty,
        //         AntBehavior::Carrying,
        //         "ant3".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(20, 5),
        //         settings.ant_color,
        //         Facing::Left,
        //         Angle::TwoHundredSeventy,
        //         AntBehavior::Carrying,
        //         "ant4".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(25, 5),
        //         settings.ant_color,
        //         Facing::Right,
        //         Angle::Zero,
        //         AntBehavior::Carrying,
        //         "ant5".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(30, 5),
        //         settings.ant_color,
        //         Facing::Right,
        //         Angle::Ninety,
        //         AntBehavior::Carrying,
        //         "ant6".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(35, 5),
        //         settings.ant_color,
        //         Facing::Right,
        //         Angle::OneHundredEighty,
        //         AntBehavior::Carrying,
        //         "ant7".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(40, 5),
        //         settings.ant_color,
        //         Facing::Right,
        //         Angle::TwoHundredSeventy,
        //         AntBehavior::Carrying,
        //         "ant8".to_string(),
        //         &asset_server,
        //     ),
        // ];

        WorldMap::new(
            settings.world_width,
            settings.world_height,
            surface_level,
            WorldSaveState {
                time_stamp: Utc::now(),
                elements: air.chain(dirt).collect(),
                ants: ants.collect(),
            },
            None,
        )
    }
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

    pub fn new(
        width: isize,
        height: isize,
        surface_level: isize,
        initial_state: WorldSaveState,
        elements: Option<Vec<Vec<Option<Entity>>>>,
    ) -> Self {
        let elements = match elements {
            Some(elements) => elements,
            None => vec![vec![None; width as usize]; height as usize],
        };

        WorldMap {
            width,
            height,
            surface_level,
            initial_state,
            elements,
        }
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    pub fn get_element(&self, position: Position) -> Option<&Entity> {
        match self
            .elements
            .get(position.y as usize)
            .and_then(|row| row.get(position.x as usize))
        {
            Some(Some(entity)) => Some(entity),
            _ => None,
        }
    }

    // TODO: Could consider returning something to prevent panic on misuse
    pub fn set_element(&mut self, position: Position, entity: Entity) {
        self.elements[position.y as usize][position.x as usize] = Some(entity);
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

static SAVE_SNAPSHOT: Mutex<Option<WorldSaveState>> = Mutex::new(None);

fn get_world_save_state(
    elements_query: &mut Query<(&Element, &Position)>,
    ants_query: &mut Query<(
        &AntOrientation,
        &AntBehavior,
        &AntTimer,
        &AntName,
        &AntColor,
        &Position,
    )>,
) -> WorldSaveState {
    let elements_save_state = elements_query
        .iter_mut()
        .map(|(element, position)| ElementSaveState {
            element: element.clone(),
            position: position.clone(),
        })
        .collect::<Vec<ElementSaveState>>();

    let ants_save_state = ants_query
        .iter_mut()
        .map(
            |(orientation, behavior, timer, name, color, position)| AntSaveState {
                orientation: orientation.clone(),
                behavior: behavior.clone(),
                timer: timer.clone(),
                name: name.clone(),
                color: color.clone(),
                position: position.clone(),
            },
        )
        .collect::<Vec<AntSaveState>>();

    WorldSaveState {
        time_stamp: Utc::now(),
        elements: elements_save_state,
        ants: ants_save_state,
    }
}

pub fn periodic_save_world_state_system(
    mut elements_query: Query<(&Element, &Position)>,
    mut ants_query: Query<(
        &AntOrientation,
        &AntBehavior,
        &AntTimer,
        &AntName,
        &AntColor,
        &Position,
    )>,
    mut last_save_time: Local<f32>,
    time: Res<Time>,
    settings: Res<Settings>,
    is_fast_forwarding: Res<IsFastForwarding>,
) {
    // Don't save while state is fast forwarding because it will cause a lot of lag.
    if is_fast_forwarding.0 {
        return;
    }

    // Limit the lifetime of the lock so that `write_save_snapshot` is able to re-acquire
    {
        let mut save_snapshot = SAVE_SNAPSHOT.lock().unwrap();
        *save_snapshot = Some(get_world_save_state(&mut elements_query, &mut ants_query));
    }

    if *last_save_time != 0.0
        && time.raw_elapsed_seconds() - *last_save_time
            < settings.auto_save_interval_ms as f32 / 1000.0
    {
        return;
    }

    if write_save_snapshot() {
        *last_save_time = time.raw_elapsed_seconds();
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
