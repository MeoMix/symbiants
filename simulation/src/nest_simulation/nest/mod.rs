use crate::{
    common::{
        ant::{
            digestion::Digestion, hunger::Hunger, NestAngle, AntBundle, AntColor, AntInventory,
            AntName, NestOrientation, AntRole, NestFacing, Initiative,
        },
        element::{Element, ElementBundle},
        grid::{ElementEntityPositionCache, Grid},
        position::Position,
        Zone,
    },
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtNest;

impl Zone for AtNest {}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Nest {
    surface_level: isize,
}

impl Nest {
    pub fn new(surface_level: isize) -> Self {
        Self { surface_level }
    }

    pub fn surface_level(&self) -> isize {
        self.surface_level
    }

    pub fn is_aboveground(&self, position: &Position) -> bool {
        !self.is_underground(position)
    }

    pub fn is_underground(&self, position: &Position) -> bool {
        position.y > self.surface_level
    }
}

pub fn register_nest(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Nest>();
    app_type_registry.write().register::<AtNest>();
}

pub fn spawn_nest(settings: Res<Settings>, mut commands: Commands) {
    let surface_level = (settings.nest_height as f32
        - (settings.nest_height as f32 * settings.initial_dirt_percent))
        as isize;

    commands.spawn((Nest::new(surface_level), AtNest));
}

// TODO: despawn_nest_elements?

/// Creates a new grid of Elements. The grid is densley populated.
/// Note the intentional omission of calling `commands.spawn_element`. This is because
/// `spawn_element` writes to the grid cache, which is not yet initialized. The grid cache will
/// be updated after this function is called. This keeps cache initialization parity between
/// creating a new world and loading an existing world.
pub fn spawn_nest_elements(
    nest_query: Query<&Nest>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let nest = nest_query.single();

    for y in 0..settings.nest_height {
        for x in 0..settings.nest_width {
            let position = Position::new(x, y);

            if y <= nest.surface_level {
                commands.spawn(ElementBundle::new(Element::Air, position, AtNest));
            } else {
                commands.spawn(ElementBundle::new(Element::Dirt, position, AtNest));
            }
        }
    }
}

pub fn spawn_nest_ants(
    nest_query: Query<&Nest>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let nest = nest_query.single();
    let mut rng = rng.reborrow();

    let queen_ant_bundle = AntBundle::new(
        // Queen always spawns in the center. She'll fall from the sky in the future.
        Position::new(settings.nest_width / 2, nest.surface_level),
        AntColor(settings.ant_color),
        AntInventory::default(),
        AntRole::Queen,
        AntName(String::from("Queen")),
        Initiative::new(&mut rng),
        AtNest,
        Hunger::new(settings.max_hunger_time),
        Digestion::new(settings.max_digestion_time),
    );

    let queen_ant_entity_id = commands.spawn(queen_ant_bundle).id();

    commands.entity(queen_ant_entity_id).insert(NestOrientation::new(
        NestFacing::random(&mut rng),
        NestAngle::Zero,
    ));

    let worker_ant_entity_ids = (0..settings.initial_ant_worker_count)
        .map(|_| {
            // TODO: maybe method on nest now
            let random_surface_position =
                Position::new(rng.isize(0..settings.nest_width), nest.surface_level);

            commands.spawn(AntBundle::new(
                random_surface_position,
                AntColor(settings.ant_color),
                AntInventory::default(),
                AntRole::Worker,
                AntName::random(&mut rng),
                Initiative::new(&mut rng),
                AtNest,
                Hunger::new(settings.max_hunger_time),
                Digestion::new(settings.max_digestion_time),
            )).id()
        })
        .collect::<Vec<_>>();

    for worker_ant_entity_id in worker_ant_entity_ids {
        commands.entity(worker_ant_entity_id).insert(NestOrientation::new(
            NestFacing::random(&mut rng),
            NestAngle::Zero,
        ));
    }
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to element entities for quick lookup outside of ECS architecture.
///
/// This is used to speed up most logic because there's a consistent need throughout the application to know what elements are
/// at or near a given position.
pub fn insert_nest_grid(
    nest_query: Query<Entity, With<Nest>>,
    element_query: Query<(&mut Position, Entity), (With<Element>, With<AtNest>)>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let mut elements_cache = vec![
        vec![Entity::PLACEHOLDER; settings.nest_width as usize];
        settings.nest_height as usize
    ];

    for (position, entity) in element_query.iter() {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    commands.entity(nest_query.single()).insert(Grid::new(
        settings.nest_width,
        settings.nest_height,
    ));

    commands.spawn((ElementEntityPositionCache(elements_cache), AtNest));
}
