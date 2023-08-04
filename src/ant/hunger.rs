use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    element::{
        is_element, commands::ElementCommandsExt, Element
    },
    map::{Position, WorldMap},
};
use super::{AntOrientation, AntInventory, Alive};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Hunger {
    value: usize,
    max: usize,
}

impl Hunger {
    pub fn default() -> Self {
        Self {
            value: 0,
            // TODO: this is 6 * 60 * 60 * 24 which is 1 day expressed in frame ticks
            max: 518400,
        }
    }

    pub fn try_increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn as_percent(&self) -> f64 {
        ((self.value as f64) / (self.max as f64) * 100.0).round()
    }

    pub fn is_hungry(&self) -> bool {
        self.value >= self.max / 2
    }

    pub fn is_starving(&self) -> bool {
        self.value >= self.max
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

pub fn ants_hunger(
    mut ants_hunger_query: Query<
        (
            Entity,
            &mut Hunger,
            &mut Handle<Image>,
            &mut AntOrientation,
            &Position,
            &AntInventory,
        ),
        With<Alive>,
    >,
    elements_query: Query<&Element>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    world_map: Res<WorldMap>,
) {
    for (entity, mut hunger, mut handle, mut orientation, position, inventory) in
        ants_hunger_query.iter_mut()
    {
        hunger.try_increment();

        // If ant is holding or adjacent to food then eat the food and reset hunger.
        if hunger.is_hungry() {
            if inventory.0 == None {
                // Check position in front of ant for food
                let food_position = *position + orientation.get_forward_delta();
                if is_element(&world_map, &elements_query, &food_position, &Element::Food) {
                    // Eat the food
                    let food_entity = world_map.get_element(food_position).unwrap();
                    info!("replace_element4: {:?}", position);

                    // TODO: command for eating food
                    commands.replace_element(food_position, *food_entity, Element::Air);

                    hunger.reset();
                }
            }
        }

        if hunger.is_starving() {
            commands.entity(entity).remove::<Alive>();

            // TODO: prefer respond to Alive removal and/or responding to addition of Dead instead of inline code here
            *handle = asset_server.load("images/ant_dead.png");
            *orientation = orientation.flip_onto_back();
        }
    }
}
