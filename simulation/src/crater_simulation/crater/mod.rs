use crate::{
    common::{
        grid::{elements_cache::ElementsCache, Grid},
        position::Position,
        Zone,
    },
    // TODO: Move most of Element and Ant to Common
    nest_simulation::{
        ant::{
            digestion::Digestion, hunger::Hunger, Angle, AntBundle, AntColor, AntInventory,
            AntName, AntOrientation, AntRole, Facing, Initiative,
        },
        element::{Element, ElementBundle},
    },
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

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
        ElementsCache::new(elements_cache),
    ));
}

/// Creates a new grid of Elements. The grid is densley populated.
/// Note the intentional omission of calling `commands.spawn_element`. This is because
/// `spawn_element` writes to the grid cache, which is not yet initialized. The grid cache will
/// be updated after this function is called. This keeps cache initialization parity between
/// creating a new world and loading an existing world.
pub fn spawn_crater_elements(settings: Res<Settings>, mut commands: Commands) {
    for y in 0..settings.crater_height {
        for x in 0..settings.crater_width {
            let position = Position::new(x, y);

            commands.spawn(ElementBundle::new(Element::Air, position, AtCrater));

            // TODO: Fill with some food at random locations.
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
        // Spawn in the center of the crater for now. In the future will need to spawn around the Nest which will be in the center.
        Position::new(settings.crater_width / 2, settings.crater_height / 2),
        AntColor(settings.ant_color),
        // TODO: No idea if this AntOrientation concept makes sense for "top-down" view, certainly wasn't designed with it in mind
        AntOrientation::new(Facing::random(&mut rng), Angle::Zero),
        AntInventory::default(),
        AntRole::Worker,
        AntName::random(&mut rng),
        Initiative::new(&mut rng),
        AtCrater,
        Hunger::new(settings.max_hunger_time),
        Digestion::new(settings.max_digestion_time),
    );

    commands.spawn(worker_ant_bundle);
}
