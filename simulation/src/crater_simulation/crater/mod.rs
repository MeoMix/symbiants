use crate::{
    common::{
        ant::{
            digestion::Digestion, hunger::Hunger, Angle, AntBundle, AntColor, AntInventory,
            AntName, AntOrientation, AntRole, Facing, Initiative,
        },
        element::{Element, ElementBundle},
        grid::{ElementEntityPositionCache, Grid},
        position::Position,
        Zone,
    },
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use super::ant::emit_pheromone::LeavingNest;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtCrater;

impl Zone for AtCrater {}

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Crater;

pub fn register_crater(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<Crater>();
    app_type_registry.write().register::<AtCrater>();
}

pub fn spawn_crater(mut commands: Commands) {
    commands.spawn((Crater, AtCrater));
}

pub fn insert_crater_grid(
    crater_query: Query<Entity, With<Crater>>,
    element_query: Query<(&mut Position, Entity), (With<Element>, With<AtCrater>)>,
    settings: Res<Settings>,
    mut commands: Commands,
) {
    let mut elements_cache = vec![
        vec![Entity::PLACEHOLDER; settings.crater_width as usize];
        settings.crater_height as usize
    ];

    for (position, entity) in element_query.iter() {
        elements_cache[position.y as usize][position.x as usize] = entity;
    }

    commands.entity(crater_query.single()).insert(Grid::new(
        settings.crater_width,
        settings.crater_height,
    ));

    commands.spawn((ElementEntityPositionCache(elements_cache), AtCrater));
}

/// Creates a new grid of Elements. The grid is densley populated.
/// Note the intentional omission of calling `commands.spawn_element`. This is because
/// `spawn_element` writes to the grid cache, which is not yet initialized. The grid cache will
/// be updated after this function is called. This keeps cache initialization parity between
/// creating a new world and loading an existing world.
pub fn spawn_crater_elements(settings: Res<Settings>, mut commands: Commands) {
    for y in 0..settings.crater_height {
        for x in 0..settings.crater_width {
            commands.spawn(ElementBundle::new(
                Element::Air,
                Position::new(x, y),
                AtCrater,
            ));
        }
    }
}

pub fn spawn_crater_ants(
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let mut rng = rng.reborrow();

    // Just spawn one worker ant for now for prototyping.
    let worker_ant_bundle = AntBundle::new(
        // Spawn adjacent to the nest entrance
        Position::new((settings.crater_width / 2) + 1, (settings.crater_height / 2) + 1),
        AntColor(settings.ant_color),
        AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
        AntInventory::default(),
        AntRole::Worker,
        AntName::random(&mut rng),
        Initiative::new(&mut rng),
        AtCrater,
        Hunger::new(settings.max_hunger_time),
        Digestion::new(settings.max_digestion_time),
    );
    let ant_entity = commands.spawn(worker_ant_bundle).id();
    commands.entity(ant_entity).insert(LeavingNest(100));
}

pub fn spawn_crater_nest() {}
