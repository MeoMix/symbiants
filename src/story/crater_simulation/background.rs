use bevy::prelude::*;

use crate::story::grid::Grid;

use super::crater::{AtCrater, Crater};

#[derive(Component)]
pub struct CraterBackground;

pub fn setup_background(mut commands: Commands, crater_query: Query<&Grid, With<Crater>>) {
    let grid = crater_query.single();

    let crater_background_sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::PURPLE,
            custom_size: Some(Vec2::new(grid.width() as f32, grid.height() as f32)),
            ..default()
        },
        ..default()
    };

    commands.spawn((crater_background_sprite, CraterBackground, AtCrater));
}

pub fn teardown_background(
    crater_background_query: Query<Entity, With<CraterBackground>>,
    mut commands: Commands,
) {
    for crater_background_entity in crater_background_query.iter() {
        commands
            .entity(crater_background_entity)
            .remove_parent()
            .despawn();
    }
}
