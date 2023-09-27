use crate::{
    ant::birthing::Birthing,
    common::register,
    element::Element,
    grid::{position::Position, WorldMap},
    settings::Settings,
};
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

use super::{commands::AntCommandsExt, AntInventory, AntOrientation, Dead, Initiative};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub enum Nesting {
    #[default]
    NotStarted,
    Started(Position),
}

pub fn initialize_nesting(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
) {
    register::<Nesting>(&app_type_registry, &mut saveable_registry);
}

pub fn ants_nesting(
    mut ants_query: Query<
        (
            &mut Nesting,
            &AntOrientation,
            &AntInventory,
            &mut Initiative,
            &Position,
            Entity,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    world_map: Res<WorldMap>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    mut commands: Commands,
) {
    for (mut nesting, orientation, inventory, mut initiative, position, ant_entity) in
        ants_query.iter_mut()
    {
        if !initiative.can_act() {
            continue;
        }

        if can_start_nesting(
            &nesting,
            &mut rng,
            &inventory,
            &position,
            &orientation,
            &world_map,
            &elements_query,
            &settings,
        ) {
            start_digging_nest(
                &position,
                &orientation,
                ant_entity,
                &mut initiative,
                &mut nesting,
                &world_map,
                &mut commands,
            );
            continue;
        }

        if can_finish_nesting(
            &position,
            &orientation,
            &world_map,
            &elements_query,
            &settings,
        ) {
            finish_digging_nest(
                &position,
                &orientation,
                ant_entity,
                &inventory,
                &mut initiative,
                &world_map,
                &mut commands,
            );
            continue;
        }
    }
}

/// Returns true if ant is at a valid location to begin digging out a nest chamber.
/// This requires six things:
///     1) The ant must not already be creating a nest.
///     2) The ant must not be carrying anything.
///     3) The ant must want to dig a nest (based on chance).
///     4) The ant must be aboveground.
///     5) The ant must not be too close to the edge of the world.
///     6) The ant must be standing on a diggable element.
/// TODO:
///     * Instead of arbitrarily checking if ant is near edge of the map, place immovable rocks which dissuade ant from digging.
fn can_start_nesting(
    nesting: &Nesting,
    rng: &mut ResMut<GlobalRng>,
    inventory: &AntInventory,
    ant_position: &Position,
    ant_orientation: &AntOrientation,
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    settings: &Settings,
) -> bool {
    let should_consider_digging = *nesting == Nesting::NotStarted
        && rng.f32() < settings.probabilities.above_surface_queen_nest_dig
        && inventory.0 == None;

    if !should_consider_digging {
        return false;
    }

    // If x position is within 20% of world edge then don't dig there
    let offset = settings.world_width / 5;
    let is_too_near_world_edge =
        ant_position.x < offset || ant_position.x > settings.world_width - offset;

    let has_valid_dig_site = world_map.is_aboveground(&ant_position) && !is_too_near_world_edge;

    let dig_position = ant_orientation.get_below_position(ant_position);
    let dig_target_entity = *world_map.element(dig_position);

    let is_element_diggable = elements_query
        .get(dig_target_entity)
        .map_or(false, |element| element.is_diggable());

    has_valid_dig_site && is_element_diggable
}

/// Start digging a nest by digging its entrance underneath the ant's current position
/// TODO:
///     * `commands.dig` could fail (conceptually, if worker ants existed) and, if it did fail, it would be wrong to mark the nest as having been started.
///     * Prefer marking nest location with pheromone rather than tracking position.
fn start_digging_nest(
    ant_position: &Position,
    ant_orientation: &AntOrientation,
    ant_entity: Entity,
    initiative: &mut Initiative,
    nesting: &mut Nesting,
    world_map: &WorldMap,
    commands: &mut Commands,
) {
    let dig_position = ant_orientation.get_below_position(ant_position);
    let dig_target_entity = *world_map.element(dig_position);
    commands.dig(ant_entity, dig_position, dig_target_entity);

    initiative.consume_action();
    *nesting = Nesting::Started(dig_position);
}

/// Returns true if ant is at a valid location to settle down and begin giving birth.
/// This requires four things:
///     1) The ant must be deep enough in the ground.
///     2) The ant must be horizontal - newborn ants shouldn't fall.
///     3) The ant must be in a spacious chamber - surrounded by air to its left/right/above.
///     4) The ant must be standing on a sturdy floor - dirt underneath it and behind it.
/// TODO:
///     * The sturdy floor check looks for Dirt, but Sand/Food/Rock is sturdy.
fn can_finish_nesting(
    ant_position: &Position,
    ant_orientation: &AntOrientation,
    world_map: &WorldMap,
    elements_query: &Query<&Element>,
    settings: &Settings,
) -> bool {
    let is_chamber_sufficiently_deep =
        ant_position.y - world_map.surface_level() > settings.minimum_nest_depth;
    if !is_chamber_sufficiently_deep {
        return false;
    }

    if !ant_orientation.is_horizontal() {
        return false;
    }

    let behind_position: Position = ant_orientation.get_behind_position(ant_position);
    let above_position = ant_orientation.get_above_position(ant_position);
    let ahead_position = ant_orientation.get_ahead_position(ant_position);

    let is_chamber_spacious = world_map.is_all_element(
        &elements_query,
        &[
            *ant_position,
            behind_position,
            above_position,
            ahead_position,
        ],
        Element::Air,
    );

    if !is_chamber_spacious {
        return false;
    }

    let below_position = ant_orientation.get_below_position(ant_position);
    let behind_below_position = below_position + ant_orientation.get_behind_delta();

    let is_chamber_floor_sturdy = world_map.is_all_element(
        &elements_query,
        &[below_position, behind_below_position],
        Element::Dirt,
    );

    if !is_chamber_floor_sturdy {
        return false;
    }

    true
}

/// Finish digging a nest by removing the Nesting instinct and adding the Birthing instinct.
/// Also, drop anything the ant is carrying so that they can eat food later.
fn finish_digging_nest(
    ant_position: &Position,
    ant_orientation: &AntOrientation,
    ant_entity: Entity,
    ant_inventory: &AntInventory,
    initiative: &mut Initiative,
    world_map: &WorldMap,
    commands: &mut Commands,
) {
    commands
        .entity(ant_entity)
        .remove::<Nesting>()
        .insert(Birthing::default());

    if ant_inventory.0 != None {
        let drop_position = ant_orientation.get_ahead_position(ant_position);
        let drop_target_entity = world_map.element(drop_position);
        commands.drop(ant_entity, drop_position, *drop_target_entity);
    }

    initiative.consume_action();
}
