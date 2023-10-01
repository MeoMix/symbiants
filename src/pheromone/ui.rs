use bevy::prelude::*;

use crate::world_map::{position::Position, WorldMap};

use super::Pheromone;

pub fn get_pheromone_sprite(pheromone: &Pheromone) -> Sprite {
    let color = match pheromone {
        Pheromone::Chamber => Color::rgba(1.0, 0.08, 0.58, 0.25),
        Pheromone::Tunnel => Color::rgba(0.0, 0.5, 0.5, 0.25),
    };

    Sprite { color, ..default() }
}

pub fn on_spawn_pheromone(
    pheromone_query: Query<
        (
            Entity,
            &Position,
            &Pheromone,
        ),
        Added<Pheromone>,
    >,
    mut commands: Commands,
    world_map: Res<WorldMap>,
) {
    for (entity, position, pheromone) in &pheromone_query {
        commands.entity(entity).insert(SpriteBundle {
            transform: Transform::from_translation(position.as_world_position(&world_map)),
            sprite: get_pheromone_sprite(pheromone),
            ..default()
        });
    }
}