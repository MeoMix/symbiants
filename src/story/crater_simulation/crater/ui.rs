use bevy::prelude::*;

use crate::story::grid::VisibleGrid;

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
        commands.entity(at_crater_entity).insert(visibility);
    }
}

pub fn on_added_crater_visible_grid(
    crater_query: Query<&Crater, Added<VisibleGrid>>,
    at_crater_query: Query<Entity, With<AtCrater>>,
    mut commands: Commands,
) {
    if crater_query.get_single().is_ok() {
        for at_crater_entity in at_crater_query.iter() {
            commands
                .entity(at_crater_entity)
                .insert(Visibility::Visible);
        }
    }
}

pub fn on_crater_removed_visible_grid(
    mut removed: RemovedComponents<VisibleGrid>,
    crater_query: Query<&Crater>,
    at_crater_query: Query<Entity, With<AtCrater>>,
    mut commands: Commands,
) {
    for entity in &mut removed {
        // If Crater was the one who had VisibleGrid removed
        if let Ok(_) = crater_query.get(entity) {
            for at_crater_entity in at_crater_query.iter() {
                commands.entity(at_crater_entity).insert(Visibility::Hidden);
            }
        }
    }
}
