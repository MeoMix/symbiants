use bevy::prelude::*;
use bevy_turborand::{GlobalRng, DelegatedRng};
use serde::{Deserialize, Serialize};

use crate::{
    story_time::StoryElapsedTicks,
    world_map::{position::Position, WorldMap}, settings::Settings, ant::emote::EmoteType,
};

use super::{AntInventory, AntOrientation, Initiative, emote::Emote};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Asleep;

pub fn ants_sleep(
    ants_query: Query<(Entity, &Position, &AntOrientation, &AntInventory), With<Initiative>>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    story_elapsed_ticks: Res<StoryElapsedTicks>,
) {
    if !story_elapsed_ticks.as_time_info().is_nighttime() {
        return;
    }

    for (ant_entity, ant_position, ant_orientation, ant_inventory) in ants_query.iter() {
        if world_map.is_underground(ant_position)
            && ant_orientation.is_rightside_up()
            && ant_inventory.0 == None
        {
            commands
                .entity(ant_entity)
                .insert(Asleep)
                .remove::<Initiative>();
        }
    }
}

// TODO: This actually seems like a view-only concern
pub fn ants_sleep_emote(
    ants_query: Query<Entity, (With<Asleep>, Without<Emote>)>,
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    settings: Res<Settings>,
) {
    for ant_entity in ants_query.iter() {
        if rng.f32() < settings.probabilities.sleep_emote {
            commands.entity(ant_entity).insert(Emote::new(EmoteType::Asleep));
        }
    }
}

pub fn ants_wake(
    ants_query: Query<Entity, With<Asleep>>,
    mut commands: Commands,
    story_elapsed_ticks: Res<StoryElapsedTicks>,
    mut rng: ResMut<GlobalRng>,
) {
    if story_elapsed_ticks.as_time_info().is_nighttime() {
        return;
    }

    for ant_entity in ants_query.iter() {
        commands
            .entity(ant_entity)
            .remove::<Asleep>()
            .insert(Initiative::new(&mut rng.reborrow()))
            // Probably not great architecture but clear the Asleep emote when waking
            .remove::<Emote>();
    }
}
