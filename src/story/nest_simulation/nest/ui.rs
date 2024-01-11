use bevy::prelude::*;

use crate::story::{grid::VisibleGrid, nest_simulation::ModelViewEntityMap};

use super::{AtNest, Nest};

// Assume for now that when the simulation loads the user wants to see their nest, but in the future might need to make this more flexible.
pub fn on_spawn_nest(nest_query: Query<Entity, Added<Nest>>, mut commands: Commands) {
    for nest_entity in nest_query.iter() {
        // TODO: Stop attaching VisibleGrid to Nest - it's a view concern not a model concern.
        commands.entity(nest_entity).insert(VisibleGrid);
    }
}

pub fn on_added_at_nest(
    nest_query: Query<&Nest, With<VisibleGrid>>,
    at_nest_model_query: Query<Entity, Added<AtNest>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    let visibility = if nest_query.get_single().is_ok() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for at_nest_model_entity in at_nest_model_query.iter() {
        if let Some(at_nest_view_entity) = model_view_entity_map.0.get(&at_nest_model_entity) {
            commands.entity(*at_nest_view_entity).insert(visibility);
        }
    }
}

pub fn on_added_nest_visible_grid(
    nest_query: Query<&Nest, Added<VisibleGrid>>,
    at_nest_model_query: Query<Entity, With<AtNest>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    if nest_query.get_single().is_ok() {
        for at_nest_model_entity in at_nest_model_query.iter() {
            if let Some(at_nest_view_entity) = model_view_entity_map.0.get(&at_nest_model_entity) {
                commands
                    .entity(*at_nest_view_entity)
                    .insert(Visibility::Visible);
            }
        }
    }
}

pub fn on_nest_removed_visible_grid(
    mut removed: RemovedComponents<VisibleGrid>,
    nest_query: Query<&Nest>,
    at_nest_model_query: Query<Entity, With<AtNest>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for entity in &mut removed.read() {
        // If Nest was the one who had VisibleGrid removed
        if let Ok(_) = nest_query.get(entity) {
            for at_nest_model_entity in at_nest_model_query.iter() {
                if let Some(at_nest_view_entity) =
                    model_view_entity_map.0.get(&at_nest_model_entity)
                {
                    commands
                        .entity(*at_nest_view_entity)
                        .insert(Visibility::Hidden);
                }
            }
        }
    }
}
