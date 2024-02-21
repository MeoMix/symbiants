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
use bevy_turborand::{DelegatedRng, GlobalRng};
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

    commands
        .entity(crater_query.single())
        .insert(Grid::new(settings.crater_width, settings.crater_height));

    commands.spawn((ElementEntityPositionCache(elements_cache), AtCrater));
}

/// Creates a new grid of Elements. The grid is densley populated.
/// Note the intentional omission of calling `commands.spawn_element`. This is because
/// `spawn_element` writes to the grid cache, which is not yet initialized. The grid cache will
/// be updated after this function is called. This keeps cache initialization parity between
/// creating a new world and loading an existing world.
pub fn spawn_crater_elements(
    settings: Res<Settings>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
) {
    // Center of the crater
    let center_x = settings.crater_width / 2;
    let center_y = settings.crater_height / 2;

    // Calculate minimum distance from center
    let min_distance_from_center = 20;

    // Adjusted ranges to ensure the food block fits within map boundaries
    let food_block_width = 10;
    let food_block_height = 5;

    let mut valid_start_position_found = false;
    let mut start_x = 0;
    let mut start_y = 0;

    // Try to find a valid starting position for the food block
    while !valid_start_position_found {
        start_x = rng.isize(0..settings.crater_width - food_block_width);
        start_y = rng.isize(0..settings.crater_height - food_block_height);

        // Calculate the distance from the center to the nearest point of the food block
        let distance_from_center = (((start_x + food_block_width / 2 - center_x).pow(2)
            + (start_y + food_block_height / 2 - center_y).pow(2))
            as f64)
            .sqrt();

        if distance_from_center >= min_distance_from_center as f64 {
            valid_start_position_found = true;
        }
    }

    // Spawn Food in a 10x5 block to ensure all food tiles are adjacent
    for y in start_y..start_y + food_block_height {
        for x in start_x..start_x + food_block_width {
            if x < settings.crater_width && y < settings.crater_height {
                commands.spawn(ElementBundle::new(
                    Element::Food,
                    Position::new(x, y),
                    AtCrater,
                ));
            }
        }
    }

    // Spawn Air everywhere food wasn't spawned
    // Note: This logic will spawn air over the entire map, potentially overwriting food
    // Consider tracking which positions have been filled with food to avoid this
    for y in 0..settings.crater_height {
        for x in 0..settings.crater_width {
            // Check if this position is outside the food block to spawn air
            if !(x >= start_x && x < start_x + 10 && y >= start_y && y < start_y + 5) {
                commands.spawn(ElementBundle::new(
                    Element::Air,
                    Position::new(x, y),
                    AtCrater,
                ));
            }
        }
    }
}

pub fn spawn_crater_ants(
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let mut rng = rng.reborrow();

    // NOTE: Just spawning some ants for prototyping
    let worker_ant_bundles = (0..10).map(|_| {
        AntBundle::new(
            // Spawn adjacent to the nest entrance
            Position::new(
                (settings.crater_width / 2) + 1,
                (settings.crater_height / 2) + 1,
            ),
            AntColor(settings.ant_color),
            AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
            AntInventory::default(),
            AntRole::Worker,
            AntName::random(&mut rng),
            Initiative::new(&mut rng),
            AtCrater,
            Hunger::new(settings.max_hunger_time),
            Digestion::new(settings.max_digestion_time),
        )
    });

    for worker_ant_bundle in worker_ant_bundles {
        let ant_entity = commands.spawn(worker_ant_bundle).id();
        commands.entity(ant_entity).insert(LeavingNest(1000.0));
    }

}

pub fn spawn_crater_nest() {}
