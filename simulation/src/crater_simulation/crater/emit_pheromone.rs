use bevy::prelude::*;

use crate::{
    common::{
        element::Food, pheromone::{commands::PheromoneCommandsExt, Pheromone, PheromoneStrength}, position::Position
    },
    settings::Settings,
};

use super::AtCrater;

// TODO: This is all written very poorly for now, just prototyping.
pub fn nest_entrance_emit_pheromone(mut commands: Commands, settings: Res<Settings>) {
    let nest_position = Position::new(settings.crater_width / 2, settings.crater_height / 2);

    commands.spawn_pheromone(
        nest_position,
        Pheromone::Nest,
        PheromoneStrength::new(50.0, 50.0),
        AtCrater,
    );
}

pub fn food_emit_pheromone(mut commands: Commands, query: Query<&Position, (With<Food>, With<AtCrater>)>) {
    for position in query.iter() {
        commands.spawn_pheromone(
            *position,
            Pheromone::Food,
            PheromoneStrength::new(50.0, 50.0),
            AtCrater,
        );
    }
}