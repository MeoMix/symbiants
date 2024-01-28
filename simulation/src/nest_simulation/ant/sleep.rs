use bevy::prelude::*;
use bevy_turborand::GlobalRng;
use serde::{Deserialize, Serialize};

use crate::{
    common::position::Position,
    nest_simulation::nest::{AtNest, Nest},
    story_time::StoryTime,
};

use super::{AntInventory, AntOrientation, Initiative};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Asleep;

pub fn ants_sleep(
    ants_query: Query<
        (Entity, &Position, &AntOrientation, &AntInventory),
        (With<Initiative>, With<AtNest>),
    >,
    mut commands: Commands,
    nest_query: Query<&Nest>,
    story_time: Res<StoryTime>,
) {
    if !story_time.is_nighttime() {
        return;
    }

    let nest = nest_query.single();

    for (ant_entity, ant_position, ant_orientation, ant_inventory) in ants_query.iter() {
        if nest.is_underground(ant_position)
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

pub fn ants_wake(
    ants_query: Query<Entity, (With<Asleep>, With<AtNest>)>,
    mut commands: Commands,
    story_time: Res<StoryTime>,
    mut rng: ResMut<GlobalRng>,
) {
    if story_time.is_nighttime() {
        return;
    }

    for ant_entity in ants_query.iter() {
        commands
            .entity(ant_entity)
            .remove::<Asleep>()
            .insert(Initiative::new(&mut rng.reborrow()));
    }
}
