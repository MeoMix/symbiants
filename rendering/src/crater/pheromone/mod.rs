use crate::common::{visible_grid::VisibleGrid, ModelViewEntityMap};
use bevy::prelude::*;
use simulation::{
    common::{
        grid::Grid,
        pheromone::{Pheromone, PheromoneStrength},
        position::Position,
    },
    crater_simulation::crater::AtCrater,
};

// #[derive(Resource)]
// pub struct PheromoneVisibility(pub Visibility);

pub fn on_spawn_pheromone(
    pheromone_query: Query<
        (Entity, &Position, &Pheromone, &PheromoneStrength),
        (Added<Pheromone>, With<AtCrater>),
    >,
    // pheromone_visibility: Res<PheromoneVisibility>,
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
            // &pheromone_visibility,
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
    // pheromone_visibility: Res<PheromoneVisibility>,
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
            // &pheromone_visibility,
            grid,
            &mut commands,
            &mut model_view_entity_map,
        );
    }
}

// pub fn on_update_pheromone_visibility(
//     pheromone_model_query: Query<Entity, With<Pheromone>>,
//     mut pheromone_view_query: Query<&mut Visibility>,
//     // pheromone_visibility: Res<PheromoneVisibility>,
//     model_view_entity_map: Res<ModelViewEntityMap>,
//     grid_query: Query<&Grid, With<AtCrater>>,
//     visible_grid: Res<VisibleGrid>,
// ) {
//     let visible_grid_entity = match visible_grid.0 {
//         Some(visible_grid_entity) => visible_grid_entity,
//         None => return,
//     };

//     if grid_query.get(visible_grid_entity).is_err() {
//         return;
//     }

//     if pheromone_visibility.is_changed() {
//         for pheromone_model_entity in pheromone_model_query.iter() {
//             if let Some(pheromone_view_entity) = model_view_entity_map.get(&pheromone_model_entity)
//             {
//                 if let Ok(mut visibility) = pheromone_view_query.get_mut(*pheromone_view_entity) {
//                     *visibility = pheromone_visibility.0;
//                 }
//             }
//         }
//     }
// }

pub fn initialize_pheromone_resources(mut commands: Commands) {
    // commands.insert_resource(PheromoneVisibility(Visibility::Visible));
}

/// Remove resources, etc.
pub fn cleanup_pheromones(mut commands: Commands) {
    // commands.remove_resource::<PheromoneVisibility>();
}

/// Non-System Helper Functions:

fn spawn_pheromone(
    pheromone_model_entity: Entity,
    pheromone: &Pheromone,
    pheromone_position: &Position,
    pheromone_strength: &PheromoneStrength,
    // pheromone_visibility: &PheromoneVisibility,
    grid: &Grid,
    commands: &mut Commands,
    model_view_entity_map: &mut ResMut<ModelViewEntityMap>,
) {
    let pheromone_view_entity = commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(
                    grid.grid_to_world_position(*pheromone_position),
                ),
                sprite: get_pheromone_sprite(pheromone, pheromone_strength),
                // visibility: pheromone_visibility.0,
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
        Pheromone::Nest => Color::rgba(1.0, 0.08, 0.58, pheromone_opacity),
        Pheromone::Food => Color::rgba(0.25, 0.88, 0.82, pheromone_opacity),
    };

    Sprite { color, ..default() }
}
