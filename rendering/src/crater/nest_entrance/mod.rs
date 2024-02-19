use bevy::prelude::*;
use simulation::{
    common::{grid::Grid, position::Position},
    crater_simulation::crater::AtCrater,
    settings::Settings,
};

use crate::common::visible_grid::grid_to_world_position;

#[derive(Component)]
pub struct NestEntrance;

pub fn spawn_nest_entrance(
    mut commands: Commands,
    settings: Res<Settings>,
    grid_query: Query<&Grid, With<AtCrater>>,
) {
    let center_position = Position::new(settings.crater_width / 2, settings.crater_height / 2);
    let grid = grid_query.single();
    let nest_entrance_sprite = SpriteBundle {
        transform: Transform::from_translation(grid_to_world_position(grid, center_position)),
        sprite: Sprite {
            color: Color::BLACK,
            // TODO: bigger nest would be good, but math is slightly harder and I am lazy
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..default()
        },
        ..default()
    };

    commands.spawn((nest_entrance_sprite, NestEntrance, AtCrater));
}

/// Remove resources, etc.
pub fn cleanup_nest_entrance() {}
