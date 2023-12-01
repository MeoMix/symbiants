use super::{
    commands::AntCommandsExt,
    digestion::Digestion,
    emote::{Emote, EmoteType},
    AntInventory, AntOrientation, AntRole, Dead, Initiative,
};
use crate::story::{
    common::position::Position,
    element::Element,
    grid::Grid,
    nest_simulation::nest::{AtNest, Nest},
    story_time::DEFAULT_TICKS_PER_SECOND,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Hunger {
    value: f32,
    max: f32,
    rate: f32,
}

impl Hunger {
    pub fn new(max_time_seconds: isize) -> Self {
        let max = 100.0;
        let rate = max / (max_time_seconds * DEFAULT_TICKS_PER_SECOND) as f32;

        Self {
            value: 0.0,
            max,
            rate,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.min(self.max).max(0.0);
    }

    pub fn tick(&mut self) {
        self.value = (self.value + self.rate).min(self.max);
    }

    pub fn is_full(&self) -> bool {
        self.value < self.max * 0.25
    }

    pub fn is_peckish(&self) -> bool {
        self.value >= self.max * 0.25
    }

    pub fn is_hungry(&self) -> bool {
        self.value >= self.max * 0.50
    }

    pub fn is_starving(&self) -> bool {
        self.value >= self.max * 0.75
    }

    pub fn is_starved(&self) -> bool {
        self.value >= self.max
    }
}

// TODO: Ants stop getting hungry while asleep which isn't really intended, but I haven't thought through Initative removal enough clearly
// because sometimes I want it for Dead + Sleep, sometimes just one or the other, and it's becoming a leaky abstraction.
pub fn ants_hunger_tick(mut ants_hunger_query: Query<&mut Hunger, (Without<Dead>, With<AtNest>)>) {
    for mut hunger in ants_hunger_query.iter_mut() {
        hunger.tick();
    }
}

// TODO: hunger should apply to ants AtCrater too
pub fn ants_hunger_act(
    mut ants_hunger_query: Query<
        (
            Entity,
            &Hunger,
            &mut Digestion,
            &AntOrientation,
            &Position,
            &mut AntInventory,
            &mut Initiative,
        ),
        With<AtNest>,
    >,
    elements_query: Query<&Element>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
) {
    let grid = nest_query.single();

    for (ant_entity, hunger, mut digestion, orientation, position, mut inventory, mut initiative) in
        ants_hunger_query.iter_mut()
    {
        if hunger.is_starved() {
            commands
                .entity(ant_entity)
                .insert(Dead)
                .remove::<Initiative>();
        } else if hunger.is_peckish() {
            if !initiative.can_act() {
                continue;
            }

            // If there is food near the hungry ant then pick it up and if the ant is holding food then eat it.
            if inventory.0 == None {
                let ahead_position = orientation.get_ahead_position(position);
                if grid
                    .elements()
                    .is_element(&elements_query, ahead_position, Element::Food)
                {
                    let food_entity = grid.elements().get_element_entity(ahead_position).unwrap();
                    commands.dig(ant_entity, ahead_position, *food_entity, AtNest);
                }
            } else {
                let element = elements_query.get(inventory.0.unwrap()).unwrap();

                if *element == Element::Food {
                    inventory.0 = None;

                    digestion.increment(-0.20);
                    initiative.consume();

                    commands
                        .entity(ant_entity)
                        .insert(Emote::new(EmoteType::FoodLove));
                }
            }
        }
    }
}

// If an ant is face-to-face with another ant then it is able to regurgitate food from itself to the other ant.
// It will only do this if the other ant is hungry.
// If the queen is starving then a worker will transfer food to it irrespective of the workers hunger level. The worker gives all it has up to 20%.
// If the other ant is hungry, then a worker will transfer food if it is well fed. This ensures workers don't spend time transferring food to a hungry ant but, in the process, make themselves hungry.

// Step 1: Find all ants which are hungry or worse.
// Step 2: For each hungry-or-worse ant, look at the position directly in front of it.
// Step 3: If there is an ant in that position, and if that ant is facing towards the hungry ant, then transfer food to the hungry ant.
pub fn ants_regurgitate(
    mut ants_hunger_query: Query<
        (
            Entity,
            &Hunger,
            &mut Digestion,
            &AntOrientation,
            &Position,
            &mut AntInventory,
            &mut Initiative,
            &mut AntRole,
        ),
        With<AtNest>,
    >,
    mut commands: Commands,
) {
    let peckish_ants = ants_hunger_query
        .iter()
        .filter(|(_, hunger, _, _, _, inventory, initiative, _)| {
            initiative.can_act() && hunger.is_peckish() && inventory.0 == None
        })
        .collect::<Vec<_>>();

    let mut results = vec![];

    for (ant_entity, ant_hunger, _, ant_orientation, ant_position, _, _, ant_role) in peckish_ants {
        let ahead_position = ant_orientation.get_ahead_position(ant_position);

        if let Some((other_ant_entity, other_ant_hunger, other_ant_digestion, _, _, _, _, _)) =
            ants_hunger_query
                .iter()
                // Support ontop of as well as in front because its kinda challenging to ensure queen can have an ant directly in front of them.
                .find(
                    |(
                        other_ant_entity,
                        _,
                        _,
                        other_ant_orientation,
                        &other_ant_position,
                        other_ant_inventory,
                        other_ant_initiative,
                        _,
                    )| {
                        if !other_ant_initiative.can_act() || other_ant_inventory.0 != None {
                            return false;
                        }

                        // If ants are adjacent and facing one another - allow regurgitation.
                        if other_ant_position == ahead_position
                            && other_ant_orientation.get_ahead_position(&other_ant_position)
                                == *ant_position
                        {
                            return true;
                        }

                        // If ants are standing ontop of one another (and not the same ant) - allow regurgitation
                        if other_ant_position == *ant_position && *other_ant_entity != ant_entity {
                            return true;
                        }

                        return false;
                    },
                )
        {
            if *ant_role == AntRole::Queen
                || (ant_hunger.is_starving() && !other_ant_hunger.is_hungry())
                || (ant_hunger.is_hungry() && other_ant_hunger.is_full())
            {
                // Transfer up to 20% digestion from other_ant to ant.
                let digestion_transfer_amount =
                    (other_ant_digestion.max() * 0.20).min(other_ant_digestion.value());

                if digestion_transfer_amount > 0.0 {
                    results.push((ant_entity, other_ant_entity, digestion_transfer_amount));
                }
            }
        }
    }

    for (ant_entity, other_ant_entity, digestion_transfer_amount) in results {
        let [(_, _, mut digestion, _, _, _, mut ant_initiative, _), (_, _, mut other_ant_digestion, _, _, _, mut other_ant_initiative, _)] =
            ants_hunger_query
                .get_many_mut([ant_entity, other_ant_entity])
                .unwrap();

        // Although initiative was checked early on in this system (when filtering entities) it's checked again here to handle an edge case of overlapping ants.
        // As an example, if there are three ants standing all in one spot, then, technically, they could all swap food.
        // However, practically, two swap food, expend their action, and the third is left without a swap partner.
        if !ant_initiative.can_act() || !other_ant_initiative.can_act() {
            continue;
        }

        digestion.value -= digestion_transfer_amount;
        other_ant_digestion.value += digestion_transfer_amount;

        ant_initiative.consume();
        other_ant_initiative.consume();

        commands
            .entity(ant_entity)
            .insert(Emote::new(EmoteType::FoodLove));
    }
}
