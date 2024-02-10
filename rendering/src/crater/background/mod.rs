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
            color: Color::Rgba { red: 0.514, green: 0.396, blue: 0.224, alpha: 1.0 },
            custom_size: Some(Vec2::new(grid.width() as f32, grid.height() as f32)),
            ..default()
        },
        ..default()
    };

    commands.spawn((crater_background_sprite, CraterBackground, AtCrater));
}

/// Remove resources, etc.
pub fn cleanup_background() {}
