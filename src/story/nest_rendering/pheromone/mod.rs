use bevy::{prelude::*, utils::HashSet};

use crate::story::{
    common::position::Position,
    grid::Grid,
    nest_simulation::nest::{AtNest, Nest},
    pheromone::{Pheromone, PheromoneStrength, PheromoneVisibility},
};

use super::common::{ModelViewEntityMap, VisibleGrid};

pub fn get_pheromone_sprite(
    pheromone: &Pheromone,
    pheromone_strength: &PheromoneStrength,
) -> Sprite {
    let pheromone_strength_opacity =
        pheromone_strength.value() as f32 / pheromone_strength.max() as f32;
    let initial_pheromone_opacity = 0.50;
    let pheromone_opacity = initial_pheromone_opacity * pheromone_strength_opacity;

    let color = match pheromone {
        Pheromone::Chamber => Color::rgba(1.0, 0.08, 0.58, pheromone_opacity),
        Pheromone::Tunnel => Color::rgba(0.25, 0.88, 0.82, pheromone_opacity),
    };

    Sprite { color, ..default() }
}

pub fn rerender_pheromones(
    pheromone_model_query: Query<(Entity, &Position, &Pheromone, &PheromoneStrength), With<AtNest>>,
    pheromone_visibility: Res<PheromoneVisibility>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    // TODO: instead of despawn could just overwrite and update?
    for (pheromone_model_entity, _, _, _) in &pheromone_model_query {
        if let Some(pheromone_view_entity) = model_view_entity_map.0.remove(&pheromone_model_entity)
        {
            commands.entity(pheromone_view_entity).despawn_recursive();
        }
    }

    let grid = nest_query.single();

    for (pheromone_model_entity, position, pheromone, pheromone_strength) in &pheromone_model_query
    {
        let pheromone_view_entity = commands
            .spawn((
                SpriteBundle {
                    transform: Transform::from_translation(grid.grid_to_world_position(*position)),
                    sprite: get_pheromone_sprite(pheromone, pheromone_strength),
                    visibility: pheromone_visibility.0,
                    ..default()
                },
                AtNest,
            ))
            .id();

        model_view_entity_map
            .0
            .insert(pheromone_model_entity, pheromone_view_entity);
    }
}

pub fn on_spawn_pheromone(
    pheromone_query: Query<
        (Entity, &Position, &Pheromone, &PheromoneStrength),
        (Added<Pheromone>, With<AtNest>),
    >,
    pheromone_visibility: Res<PheromoneVisibility>,
    mut commands: Commands,
    nest_query: Query<&Grid, With<Nest>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match nest_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (pheromone_model_entity, position, pheromone, pheromone_strength) in &pheromone_query {
        let pheromone_view_entity = commands
            .spawn((
                SpriteBundle {
                    transform: Transform::from_translation(grid.grid_to_world_position(*position)),
                    sprite: get_pheromone_sprite(pheromone, pheromone_strength),
                    visibility: pheromone_visibility.0,
                    ..default()
                },
                AtNest,
            ))
            .id();

        model_view_entity_map
            .0
            .insert(pheromone_model_entity, pheromone_view_entity);
    }
}

pub fn on_despawn_pheromone(
    mut removed: RemovedComponents<Pheromone>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let model_entities = &mut removed.read().collect::<HashSet<_>>();

    for model_entity in model_entities.iter() {
        if let Some(view_entity) = model_view_entity_map.0.remove(model_entity) {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}

pub fn on_update_pheromone_visibility(
    pheromone_model_query: Query<Entity, With<Pheromone>>,
    mut pheromone_view_query: Query<&mut Visibility>,
    pheromone_visibility: Res<PheromoneVisibility>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    nest_query: Query<&Grid, With<Nest>>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if nest_query.get(visible_grid_entity).is_err() {
        return;
    }

    if pheromone_visibility.is_changed() {
        for pheromone_model_entity in pheromone_model_query.iter() {
            if let Some(pheromone_view_entity) =
                model_view_entity_map.0.get(&pheromone_model_entity)
            {
                if let Ok(mut visibility) = pheromone_view_query.get_mut(*pheromone_view_entity) {
                    *visibility = pheromone_visibility.0;
                }
            }
        }
    }
}

pub fn teardown_pheromone(
    pheromone_model_query: Query<Entity, With<Pheromone>>,
    mut commands: Commands,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    for pheromone_model_entity in pheromone_model_query.iter() {
        if let Some(pheromone_view_entity) = model_view_entity_map.0.remove(&pheromone_model_entity)
        {
            commands.entity(pheromone_view_entity).despawn_recursive();
        }
    }
}
