use super::{commands::AntCommandsExt, AntInventory, AntOrientation, Dead, Initiative};
use crate::{
    common::{get_entity_from_id, Id},
    element::Element,
    grid::{position::Position, WorldMap},
    story_state::StoryState,
    time::{DEFAULT_SECONDS_PER_TICK, SECONDS_PER_DAY},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Hunger {
    value: f32,
    max: f32,
    rate_of_hunger: f32,
}

impl Hunger {
    pub fn default() -> Self {
        let max = 100.0;
        let rate_of_hunger = max / (SECONDS_PER_DAY as f32 * DEFAULT_SECONDS_PER_TICK);

        Self {
            value: 0.0,
            max,
            rate_of_hunger,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn tick(&mut self) {
        self.value = (self.value + self.rate_of_hunger).min(self.max);
    }

    pub fn is_hungry(&self) -> bool {
        self.value >= self.max / 2.0
    }

    pub fn is_starving(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

pub fn ants_hunger(
    mut ants_hunger_query: Query<
        (
            Entity,
            &mut Hunger,
            &mut AntOrientation,
            &Position,
            &mut AntInventory,
            &mut Initiative,
        ),
        Without<Dead>,
    >,
    elements_query: Query<&Element>,
    id_query: Query<(Entity, &Id)>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
    mut story_state: ResMut<NextState<StoryState>>,
) {
    for (entity, mut hunger, mut orientation, position, mut inventory, mut initiative) in
        ants_hunger_query.iter_mut()
    {
        hunger.tick();

        if hunger.is_starving() {
            commands.entity(entity).insert(Dead);
            *orientation = orientation.flip_onto_back();
            // NOTE: It's unfortunate I set this here and then duplicate logic in setup_story_state
            story_state.set(StoryState::Over);
        } else if hunger.is_hungry() {
            if !initiative.can_act() {
                continue;
            }

            // If there is food near the hungry ant then pick it up and if the ant is holding food then eat it.
            if inventory.0 == None {
                let food_position = *position + orientation.get_forward_delta();
                if world_map.is_element(&elements_query, food_position, Element::Food) {
                    let food_entity = world_map.get_element(food_position).unwrap();
                    commands.dig(entity, food_position, *food_entity);
                    initiative.consume_action();
                }
            } else {
                let id = inventory.0.clone().unwrap();
                let entity = get_entity_from_id(id, &id_query).unwrap();
                let element = elements_query.get(entity).unwrap();

                if *element == Element::Food {
                    inventory.0 = None;
                    hunger.reset();
                    initiative.consume_action();
                }
            }
        }
    }
}
