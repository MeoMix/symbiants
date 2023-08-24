use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use std::ops::Add;

use crate::{
    ant::{
        Angle, AntBundle, AntColor, AntInventory, AntName, AntOrientation, AntRole, Facing,
        Initiative,
    },
    common::Id,
    element::{AirElementBundle, DirtElementBundle, Element},
    food::FoodCount,
    name_list::get_random_name,
    nest::Nest,
    settings::Settings,
    time::GameTime,
};

pub mod position;
pub mod save;

use chrono::{DateTime, Utc};

use self::{position::Position, save::load_existing_world};

// TODO: Maybe parts of this should be reflected rather than regenerating from settings?
#[derive(Resource, Debug)]
pub struct WorldMap {
    width: isize,
    height: isize,
    surface_level: isize,
    created_at: DateTime<Utc>,
    elements_cache: Vec<Vec<Entity>>,
}

pub fn setup_world_map(world: &mut World) {
    // TODO: I'm reusing this setup for both initial app load and reload of app state but should probably split them apart?
    // Clearing persistent entities is only needed for reload of app state.
    // Checking `load_existing_world` isn't necessary when reloading, too.
    let mut persistent_entity_query = world.query_filtered::<Entity, With<Id>>();
    let persistent_entites = persistent_entity_query.iter(&world).collect::<Vec<_>>();

    for entity in persistent_entites {
        world.entity_mut(entity).despawn_recursive();
    }

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
    world.init_resource::<GameTime>();
    world.init_resource::<Nest>();

    for y in 0..settings.world_height {
        for x in 0..settings.world_width {
            let position = Position::new(x, y);

            if y <= settings.get_surface_level() {
                world.spawn(AirElementBundle::new(position));
            } else {
                world.spawn(DirtElementBundle::new(position));
            }
        }
    }

    let ants = {
        let mut rng = world.resource_mut::<GlobalRng>();

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

    pub fn element(&self, position: Position) -> &Entity {
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
