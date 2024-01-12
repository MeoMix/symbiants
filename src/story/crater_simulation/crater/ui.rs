use bevy::prelude::*;

use crate::story::{grid::VisibleGrid, common::ui::ModelViewEntityMap};

use super::{AtCrater, Crater};

pub fn on_added_at_crater(
    crater_query: Query<&Crater, With<VisibleGrid>>,
    at_crater_query: Query<Entity, Added<AtCrater>>,
    mut commands: Commands,
) {
    let visibility = if crater_query.get_single().is_ok() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for at_crater_entity in at_crater_query.iter() {
        // TODO: Stop attaching VisibleGrid to Crater - it's a view concern not a model concern.
        commands.entity(at_crater_entity).insert(visibility);
    }
}

pub fn on_added_crater_visible_grid(
    crater_query: Query<&Crater, Added<VisibleGrid>>,
    at_crater_model_query: Query<Entity, With<AtCrater>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    if crater_query.get_single().is_ok() {
        for at_crater_model_entity in at_crater_model_query.iter() {
            if let Some(at_crater_view_entity) =
                model_view_entity_map.0.get(&at_crater_model_entity)
            {
                commands
                    .entity(*at_crater_view_entity)
                    .insert(Visibility::Visible);
            }
        }
    }
}

pub fn on_crater_removed_visible_grid(
    mut removed: RemovedComponents<VisibleGrid>,
    crater_query: Query<&Crater>,
    at_crater_model_query: Query<Entity, With<AtCrater>>,
    mut commands: Commands,
    model_view_entity_map: Res<ModelViewEntityMap>,
) {
    for entity in &mut removed.read() {
        // If Crater was the one who had VisibleGrid removed
        if let Ok(_) = crater_query.get(entity) {
            for at_crater_model_entity in at_crater_model_query.iter() {
                if let Some(at_crater_view_entity) =
                    model_view_entity_map.0.get(&at_crater_model_entity)
                {
                    commands
                        .entity(*at_crater_view_entity)
                        .insert(Visibility::Hidden);
                }
            }
        }
    }
}
