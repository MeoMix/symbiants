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
    Nest,
    Crater,
}

pub fn grid_to_tile_pos(grid: &Grid, position: Position) -> TilePos {
    TilePos {
        x: position.x as u32,
        y: (grid.height() - position.y - 1) as u32,
    }
}
