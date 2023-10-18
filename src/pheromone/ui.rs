use bevy::prelude::*;

use crate::world_map::{position::Position, WorldMap};

use super::{Pheromone, PheromoneVisibility, PheromoneStrength};

pub fn get_pheromone_sprite(pheromone: &Pheromone, pheromone_strength: &PheromoneStrength) -> Sprite {
    let pheromone_strength_opacity = pheromone_strength.value() as f32 / pheromone_strength.max() as f32;
    let initial_pheromone_opacity = 0.50;
    let pheromone_opacity = initial_pheromone_opacity * pheromone_strength_opacity;

    let color = match pheromone {
        Pheromone::Chamber => Color::rgba(1.0, 0.08, 0.58, pheromone_opacity),
        Pheromone::Tunnel => Color::rgba(0.0, 0.5, 0.5, pheromone_opacity),
    };

    Sprite { color, ..default() }
}

pub fn on_spawn_pheromone(
    pheromone_query: Query<(Entity, &Position, &Pheromone, &PheromoneStrength), Added<Pheromone>>,
    pheromone_visibility: Res<PheromoneVisibility>,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, position, pheromone, pheromone_strength) in &pheromone_query {
        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(position.as_world_position(&world_map)),
            sprite: get_pheromone_sprite(pheromone, pheromone_strength),
            visibility: pheromone_visibility.0,
            ..default()
        });
    }
}

pub fn on_update_pheromone_visibility(
    mut pheromone_query: Query<&mut Visibility, With<Pheromone>>,
    pheromone_visibility: Res<PheromoneVisibility>,
) {
    if pheromone_visibility.is_changed() {
        for mut visibility in pheromone_query.iter_mut() {
            *visibility = pheromone_visibility.0;
        }
    }
}