use super::{commands::AntCommandsExt, Dead, AntInventory, AntOrientation, Initiative};
use crate::{
    element::Element,
    map::{Position, WorldMap},
    time::{DEFAULT_TICK_RATE, SECONDS_PER_DAY},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Hunger {
    value: f32,
    max: f32,
    rate_of_hunger: f32,
}

impl Hunger {
    pub fn default() -> Self {
        let max = 100.0;
        let rate_of_hunger = max / (SECONDS_PER_DAY as f32 * DEFAULT_TICK_RATE);

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
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, mut hunger, mut orientation, position, mut inventory, mut initiative) in
        ants_hunger_query.iter_mut()
    {
        hunger.tick();

        if hunger.is_starving() {
            commands.entity(entity).insert(Dead);
            *orientation = orientation.flip_onto_back();
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
                }
                initiative.act();
            } else if inventory.0 == Some(Element::Food) {
                inventory.0 = Some(Element::Air);
                hunger.reset();
                initiative.act();
            }
        }
    }
}
