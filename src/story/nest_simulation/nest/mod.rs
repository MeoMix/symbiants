use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

pub mod ui;

use crate::{
    settings::Settings,
    story::{
        ant::{
            commands::AntCommandsExt, Angle, AntColor, AntInventory, AntName, AntOrientation,
            AntRole, Facing, Initiative,
        },
        common::{position::Position, register, Id, Location},
        element::{Element, ElementBundle},
        grid::{elements_cache::ElementsCache, Grid},
    },
};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtNest;

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

pub fn register_nest(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Nest>(&app_type_registry, &mut saveable_registry);
    register::<AtNest>(&app_type_registry, &mut saveable_registry);
}

pub fn setup_nest(settings: Res<Settings>, mut commands: Commands) {
    commands.spawn((Nest::new(settings.get_surface_level()), Id::default()));
}

pub fn setup_nest_elements(settings: Res<Settings>, mut commands: Commands) {
    for y in 0..settings.nest_height {
        for x in 0..settings.nest_width {
            let position = Position::new(x, y);

            // FIXME: These should be commands.spawn_element but need to fix circularity with expecting Nest to exist.
            if y <= settings.get_surface_level() {
                commands.spawn(ElementBundle::new(Element::Air, position, Location::Nest));
            } else {
                commands.spawn(ElementBundle::new(Element::Dirt, position, Location::Nest));
            }
        }
    }
}

pub fn setup_nest_ants(
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let mut rng = rng.reborrow();

    // Newly created queens instinctively start building a nest.
    commands.spawn_ant(
        // Queen always spawns in the center. She'll fall from the sky in the future.
        Position::new(settings.nest_width / 2, settings.get_surface_level()),
        AntColor(settings.ant_color),
        AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
        AntInventory::default(),
        AntRole::Queen,
        AntName(String::from("Queen")),
        Initiative::new(&mut rng),
        Location::Nest,
    );

    for _ in 0..settings.initial_ant_worker_count {
        // TODO: Prefer spawn_batch but would need to create custom command for spawning batch ants
        commands.spawn_ant(
            settings.get_random_surface_position(&mut rng),
            AntColor(settings.ant_color),
            AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
            AntInventory::default(),
            AntRole::Worker,
            AntName::random(&mut rng),
            Initiative::new(&mut rng),
            Location::Nest,
        );
    }
}

/// Called after creating a new story, or loading an existing story from storage.
/// Creates a cache that maps positions to element entities for quick lookup outside of ECS architecture.
///
/// This is used to speed up most logic because there's a consistent need throughout the application to know what elements are
/// at or near a given position.
pub fn setup_nest_grid(
    nest_query: Query<Entity, With<Nest>>,
    element_query: Query<(&mut Position, Entity), With<Element>>,
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

    commands.entity(nest_query.single()).insert((Grid::new(
        settings.nest_width,
        settings.nest_height,
        ElementsCache::new(elements_cache),
    ),));
}

pub fn teardown_nest(mut commands: Commands, nest_entity_query: Query<Entity, With<Nest>>) {
    let nest_entity = nest_entity_query.single();

    commands.entity(nest_entity).despawn();
}
