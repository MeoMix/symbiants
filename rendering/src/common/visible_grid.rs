use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use simulation::common::{grid::Grid, position::Position};

#[derive(Resource, Default)]
pub struct VisibleGrid(pub Option<Entity>);

// TODO: It's weird that I have the concept of `VisibleGrid` in addition to `VisibleGridState`
// Generally representing the same state in two different ways is a great way to introduce bugs.
#[derive(States, Default, Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum VisibleGridState {
    #[default]
    None,
    Nest,
    Crater,
}

pub fn grid_to_tile_pos(grid: &Grid, position: Position) -> TilePos {
    TilePos {
        x: position.x as u32,
        y: (grid.height() - position.y - 1) as u32,
    }
}

// Convert Position to Transform, z-index is naively set to 1 for now
pub fn grid_to_world_position(grid: &Grid, position: Position) -> Vec3 {
    let y_offset = grid.height() as f32 / 2.0;
    let x_offset = grid.width() as f32 / 2.0;

    Vec3 {
        // NOTE: unit width is 1.0 so add 0.5 to center the position
        x: position.x as f32 - x_offset + 0.5,
        // The view of the model position is just an inversion along the y-axis.
        y: -position.y as f32 + y_offset - 0.5,
        z: 1.0,
    }
}

pub fn world_to_grid_position(grid: &Grid, world_position: Vec2) -> Position {
    let x = world_position.x + (grid.width() as f32 / 2.0) - 0.5;
    let y = -world_position.y + (grid.height() as f32 / 2.0) - 0.5;

    Position {
        x: x.abs().round() as isize,
        y: y.abs().round() as isize,
    }
}

pub fn set_visible_grid_state_none(
    mut next_visible_grid_state: ResMut<NextState<VisibleGridState>>,
) {
    next_visible_grid_state.set(VisibleGridState::None);
}

pub fn set_visible_grid_state_nest(
    mut next_visible_grid_state: ResMut<NextState<VisibleGridState>>,
) {
    next_visible_grid_state.set(VisibleGridState::Nest);
}
