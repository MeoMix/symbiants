use crate::common::{
    pheromone::PheromoneVisibility,
    visible_grid::{grid_to_world_position, VisibleGrid},
    ModelViewEntityMap,
};
use bevy::prelude::*;
use simulation::{
    common::{
        grid::Grid,
        pheromone::{Pheromone, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

pub fn on_spawn_pheromone(
    pheromone_query: Query<
        (Entity, &Position, &Pheromone, &PheromoneStrength),
        (Added<Pheromone>, With<AtCrater>),
    >,
    pheromone_visibility: Res<PheromoneVisibility>,
    mut commands: Commands,
    grid_query: Query<&Grid, With<AtCrater>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    let grid = match grid_query.get(visible_grid_entity) {
        Ok(grid) => grid,
        Err(_) => return,
    };

    for (pheromone_model_entity, position, pheromone, pheromone_strength) in &pheromone_query {
        spawn_pheromone(
            pheromone_model_entity,
            pheromone,
            position,
            pheromone_strength,
            &pheromone_visibility,
            grid,
            &mut commands,
            &mut model_view_entity_map,
        );
    }
}

pub fn spawn_pheromones(
    pheromone_model_query: Query<
        (Entity, &Position, &Pheromone, &PheromoneStrength),
        With<AtCrater>,
    >,
    pheromone_visibility: Res<PheromoneVisibility>,
    mut commands: Commands,
    grid_query: Query<&Grid, With<AtCrater>>,
    mut model_view_entity_map: ResMut<ModelViewEntityMap>,
) {
    let grid = grid_query.single();

    for (pheromone_model_entity, position, pheromone, pheromone_strength) in &pheromone_model_query
    {
        spawn_pheromone(
            pheromone_model_entity,
            pheromone,
            position,
            pheromone_strength,
            &pheromone_visibility,
            grid,
            &mut commands,
            &mut model_view_entity_map,
        );
    }
}

pub fn on_update_pheromone_strength(
    pheromone_model_query: Query<(Entity, &Pheromone, Ref<PheromoneStrength>), With<AtCrater>>,
    mut sprite_query: Query<&mut Sprite>,
    model_view_entity_map: Res<ModelViewEntityMap>,
    visible_grid: Res<VisibleGrid>,
    grid_query: Query<&Grid, With<AtCrater>>,
) {
    let visible_grid_entity = match visible_grid.0 {
        Some(visible_grid_entity) => visible_grid_entity,
        None => return,
    };

    if grid_query.get(visible_grid_entity).is_err() {
        return;
    }

    for (pheromone_model_entity, pheromone, pheromone_strength) in pheromone_model_query.iter() {
        if !pheromone_strength.is_changed() || pheromone_strength.is_added() {
            continue;
        }

        if let Some(pheromone_view_entity) = model_view_entity_map.get(&pheromone_model_entity) {
            let mut sprite = sprite_query.get_mut(*pheromone_view_entity).unwrap();
            *sprite = get_pheromone_sprite(pheromone, pheromone_strength.as_ref());
        }
    }
}

/// Non-System Helper Functions:

fn spawn_pheromone(
    pheromone_model_entity: Entity,
    pheromone: &Pheromone,
    pheromone_position: &Position,
    pheromone_strength: &PheromoneStrength,
    pheromone_visibility: &PheromoneVisibility,
    grid: &Grid,
    commands: &mut Commands,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    let pheromone_view_entity = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(grid_to_world_position(
                    grid,
                    *pheromone_position,
                )),
                sprite: get_pheromone_sprite(pheromone, pheromone_strength),
                visibility: pheromone_visibility.0,
                ..default()
            },
            AtCrater,
        ))
        .id();

    model_view_entity_map.insert(pheromone_model_entity, pheromone_view_entity);
}

fn get_pheromone_sprite(pheromone: &Pheromone, pheromone_strength: &PheromoneStrength) -> Sprite {
    let pheromone_strength_opacity =
        pheromone_strength.value() as f32 / pheromone_strength.max() as f32;
    let initial_pheromone_opacity = 0.50;
    let pheromone_opacity = initial_pheromone_opacity * pheromone_strength_opacity;

    let color = match pheromone {
        Pheromone::Chamber => panic!("not supported"),
        Pheromone::Tunnel => panic!("not supported"),
        // TODO: better colors
        Pheromone::Nest => Color::srgba(1.0, 0.08, 0.58, pheromone_opacity),
        Pheromone::Food => Color::srgba(0.25, 0.88, 0.82, pheromone_opacity),
    };

    Sprite { color, ..default() }
}
