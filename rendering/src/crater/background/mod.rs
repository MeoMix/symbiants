use bevy::prelude::*;
use simulation::{
    common::grid::Grid,
    crater_simulation::crater::{AtCrater, Crater},
};

#[derive(Component)]
pub struct CraterBackground;

pub fn spawn_background(mut commands: Commands, crater_query: Query<&Grid, With<Crater>>) {
    let grid = crater_query.single();

    let crater_background_sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::BEIGE,
            custom_size: Some(Vec2::new(grid.width() as f32, grid.height() as f32)),
            ..default()
        },
        ..default()
    };

    commands.spawn((crater_background_sprite, CraterBackground, AtCrater));
}

pub fn cleanup_background(
    crater_background_query: Query<Entity, With<CraterBackground>>,
    mut commands: Commands,
) {
    let crater_background_entity = crater_background_query.single();
    commands.entity(crater_background_entity).despawn();
}
