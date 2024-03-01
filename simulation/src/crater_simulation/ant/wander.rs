use crate::{
    common::{
        ant::{initiative::Initiative, CraterOrientation},
        element::Element,
        grid::GridElements,
        position::Position,
    },
    crater_simulation::crater::AtCrater,
    settings::Settings,
};
use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng};

/// Wandering is a low-priority task which occurs if ants aren't following pheromones.
/// Ant will generally try to walk forward unless it is blocked. If it's blocked, or by chance,
/// then it will turn left/right.
pub fn ants_wander(
    mut ants_query: Query<(&mut Initiative, &mut Position, &mut CraterOrientation), With<AtCrater>>,
    settings: Res<Settings>,
    mut rng: ResMut<GlobalRng>,
    grid_elements: GridElements<AtCrater>,
) {
    for (mut initiative, mut position, mut orientation) in ants_query.iter_mut() {
        if !initiative.can_move() {
            continue;
        }

        let ahead_position = orientation.get_ahead_position(&position);
        let has_air_ahead = grid_elements
            .get_entity(ahead_position)
            .map_or(false, |entity| {
                *grid_elements.element(*entity) == Element::Air
            });

        let is_turning_randomly = rng.chance(settings.probabilities.random_crater_turn.into());

        if !has_air_ahead || is_turning_randomly {
            *orientation = *rng.sample(&orientation.get_perpendicular()).unwrap();
        } else {
            *position = ahead_position;
        }

        initiative.consume_movement();
    }
}
