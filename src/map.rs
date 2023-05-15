use bevy::{prelude::*, utils::HashMap};
use gloo_storage::{LocalStorage, Storage};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Deref, Mul},
    sync::Mutex,
};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::{
    ant::{AntAngle, AntBehavior, AntColor, AntFacing, AntName, AntSaveState},
    elements::{Element, ElementSaveState},
    name_list::NAMES,
    settings::Settings,
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
}

impl Add for Position {
    type Output = Self;

    // TODO: Hexx uses const_add here?
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
    width: isize,
    height: isize,
    surface_level: isize,
    // TODO: Should not have this be public
    pub initial_state: WorldSaveState,
    pub elements: HashMap<Position, Entity>,
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

        // if let Ok(saved_state) = LocalStorage::get::<WorldSaveState>(LOCAL_STORAGE_KEY) {
        //     return WorldMap::new(
        //         settings.world_width,
        //         settings.world_height,
        //         surface_level,
        //         saved_state,
        //         None,
        //     );
        // }

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
            let x = world_rng.rng.gen_range(0..1000) % settings.world_width;
            // Put the ant on the dirt.
            let y = surface_level;

            // Randomly position ant facing left or right.
            let facing = if world_rng.rng.gen_bool(0.5) {
                AntFacing::Left
            } else {
                AntFacing::Right
            };

            let name = NAMES[world_rng.rng.gen_range(0..NAMES.len())].clone();

            AntSaveState {
                position: Position::new(x, y),
                color: AntColor(settings.ant_color),
                facing,
                angle: AntAngle::Zero,
                behavior: AntBehavior::Wandering,
                name: AntName(name.to_string()),
            }
        });

        // let ants = [
        //     AntSaveState {
        //         position: Position::new(5, 5),
        //         color: settings.ant_color,
        //         facing: AntFacing::Left,
        //         angle: AntAngle::Zero,
        //         behavior: AntBehavior::Carrying,
        //         name: "ant1".to_string(),
        //     },
        //     Ant::new(
        //         Position::new(10, 5),
        //         settings.ant_color,
        //         AntFacing::Left,
        //         AntAngle::Ninety,
        //         AntBehavior::Carrying,
        //         "ant2".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(15, 5),
        //         settings.ant_color,
        //         AntFacing::Left,
        //         AntAngle::OneHundredEighty,
        //         AntBehavior::Carrying,
        //         "ant3".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(20, 5),
        //         settings.ant_color,
        //         AntFacing::Left,
        //         AntAngle::TwoHundredSeventy,
        //         AntBehavior::Carrying,
        //         "ant4".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(25, 5),
        //         settings.ant_color,
        //         AntFacing::Right,
        //         AntAngle::Zero,
        //         AntBehavior::Carrying,
        //         "ant5".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(30, 5),
        //         settings.ant_color,
        //         AntFacing::Right,
        //         AntAngle::Ninety,
        //         AntBehavior::Carrying,
        //         "ant6".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(35, 5),
        //         settings.ant_color,
        //         AntFacing::Right,
        //         AntAngle::OneHundredEighty,
        //         AntBehavior::Carrying,
        //         "ant7".to_string(),
        //         &asset_server,
        //     ),
        //     Ant::new(
        //         Position::new(40, 5),
        //         settings.ant_color,
        //         AntFacing::Right,
        //         AntAngle::TwoHundredSeventy,
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
        elements: Option<HashMap<Position, Entity>>,
    ) -> Self {
        WorldMap {
            width,
            height,
            surface_level,
            initial_state,
            elements: elements.unwrap_or_default(),
        }
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }
}

pub fn setup_window_onunload_save_world_state() {
    let window = web_sys::window().expect("window not available");

    let on_beforeunload = Closure::wrap(Box::new(move |_event: web_sys::BeforeUnloadEvent| {
        let save_snapshot = SAVE_SNAPSHOT.lock().unwrap();
        let save_result = LocalStorage::set(LOCAL_STORAGE_KEY, save_snapshot.deref().clone());

        if save_result.is_err() {
            error!(
                "Failed to save world state to local storage: {:?}",
                save_result
            );
        }
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

pub fn save_snapshot_system(
    mut elements_query: Query<(&Element, &Position)>,
    mut ants_query: Query<(
        &AntFacing,
        &AntAngle,
        &AntBehavior,
        &AntName,
        &AntColor,
        &Position,
    )>,
) {
    // TODO: don't copy/paste this code
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
            |(facing, angle, behavior, name, color, position)| AntSaveState {
                facing: facing.clone(),
                angle: angle.clone(),
                behavior: behavior.clone(),
                name: name.clone(),
                color: color.clone(),
                position: position.clone(),
            },
        )
        .collect::<Vec<AntSaveState>>();

    {
        let mut save_snapshot = SAVE_SNAPSHOT.lock().unwrap();

        *save_snapshot = Some(WorldSaveState {
            time_stamp: Utc::now(),
            elements: elements_save_state,
            ants: ants_save_state,
        });
    }
}

fn save_world_state(
    elements_query: &mut Query<(&Element, &Position)>,
    ants_query: &mut Query<(
        &AntFacing,
        &AntAngle,
        &AntBehavior,
        &AntName,
        &AntColor,
        &Position,
    )>,
) {
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
            |(facing, angle, behavior, name, color, position)| AntSaveState {
                facing: facing.clone(),
                angle: angle.clone(),
                behavior: behavior.clone(),
                name: name.clone(),
                color: color.clone(),
                position: position.clone(),
            },
        )
        .collect::<Vec<AntSaveState>>();

    let result = LocalStorage::set::<WorldSaveState>(
        LOCAL_STORAGE_KEY.to_string(),
        WorldSaveState {
            time_stamp: Utc::now(),
            elements: elements_save_state,
            ants: ants_save_state,
        },
    );

    if result.is_err() {
        error!("Failed to save world state: {:?}", result);
    }
}

pub fn periodic_save_world_state_system(
    mut elements_query: Query<(&Element, &Position)>,
    mut ants_query: Query<(
        &AntFacing,
        &AntAngle,
        &AntBehavior,
        &AntName,
        &AntColor,
        &Position,
    )>,
    mut last_save_time: Local<f32>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    // TODO: don't run this when fast forwarding

    if *last_save_time != 0.0
        && time.raw_elapsed_seconds() - *last_save_time
            < settings.auto_save_interval_ms as f32 / 1000.0
    {
        return;
    }

    save_world_state(&mut elements_query, &mut ants_query);

    *last_save_time = time.raw_elapsed_seconds();
}