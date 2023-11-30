pub mod ui;

use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use crate::{
    settings::Settings,
    story::{
        ant::{
            digestion::Digestion, hunger::Hunger, Angle, AntBundle, AntColor, AntInventory,
            AntName, AntOrientation, AntRole, Facing, Initiative,
        },
        common::{position::Position, register, Id, Zone},
        element::{Element, ElementBundle},
        grid::{elements_cache::ElementsCache, Grid},
    },
};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct AtCrater;

impl Zone for AtCrater {}

/// Note the intentional omission of reflection/serialization.
/// This is because Crater is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Crater;

pub fn register_crater(app_type_registry: ResMut<AppTypeRegistry>) {
    register::<Crater>(&app_type_registry);
    register::<AtCrater>(&app_type_registry);
}

pub fn setup_crater(mut commands: Commands) {
    commands.spawn((Crater, AtCrater, Id::default()));
}

/// Creates a new grid of Elements. The grid is densley populated.
/// Note the intentional omission of calling `commands.spawn_element`. This is because
/// `spawn_element` writes to the grid cache, which is not yet initialized. The grid cache will
/// be updated after this function is called. This keeps cache initialization parity between
/// creating a new world and loading an existing world.
pub fn setup_crater_elements(settings: Res<Settings>, mut commands: Commands) {
    for y in 0..settings.crater_height {
        for x in 0..settings.crater_width {
            let position = Position::new(x, y);
            commands.spawn(ElementBundle::new(Element::Air, position, AtCrater));
        }
    }
}

pub fn setup_crater_ants(
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    let mut rng = rng.reborrow();

    let worker_ant_bundles = (0..1)
        .map(|_| {
            let center_crater_position =
                Position::new(settings.crater_width / 2, settings.crater_height / 2);

            AntBundle::new(
                center_crater_position,
                AntColor(settings.ant_color),
                // TODO: not positive AntOrientation makes sense in the context of Crater but going to try for now
                AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
                AntInventory::default(),
                AntRole::Worker,
                AntName::random(&mut rng),
                Initiative::new(&mut rng),
                AtCrater,
                Hunger::new(settings.max_hunger_time),
                Digestion::new(settings.max_digestion_time),
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(worker_ant_bundles)
}

pub fn setup_crater_grid(
    element_query: Query<(&mut Position, Entity), (With<Element>, With<AtCrater>)>,
    crater_query: Query<Entity, With<Crater>>,
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

    commands.entity(crater_query.single()).insert((Grid::new(
        settings.crater_width,
        settings.crater_height,
        ElementsCache::new(elements_cache),
    ),));
}

pub fn teardown_crater(mut commands: Commands, crater_entity_query: Query<Entity, With<Crater>>) {
    let crater_entity = crater_entity_query.single();

    commands.entity(crater_entity).despawn();
}
