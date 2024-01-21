pub mod elements_cache;

use bevy::prelude::*;

use crate::story::common::position::Position;

use self::elements_cache::ElementsCache;

// TODO: I think this is a view concern...?
// TODO: prob don't want both component (VisibleGrid) and VisibleGridState? idk
#[derive(States, Default, Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub enum VisibleGridState {
    #[default]
    Nest,
    Crater,
}

/// Note the intentional omission of reflection/serialization.
/// This is because Grid is a cache that is trivially regenerated on app startup from persisted state.
#[derive(Component, Debug)]
pub struct Grid {
    width: isize,
    height: isize,
    elements_cache: ElementsCache,
}

impl Grid {
    pub fn new(width: isize, height: isize, elements_cache: ElementsCache) -> Self {
        Self {
            width,
            height,
            elements_cache,
        }
    }

    pub fn width(&self) -> isize {
        self.width
    }

    pub fn height(&self) -> isize {
        self.height
    }

    pub fn elements(&self) -> &ElementsCache {
        &self.elements_cache
    }

    pub fn elements_mut(&mut self) -> &mut ElementsCache {
        &mut self.elements_cache
    }

    pub fn is_within_bounds(&self, position: &Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }

    // TODO: This still isn't the right spot for it I think, but living here for now. Maybe move into a dedicate UI layer later on
    // Convert Position to Transform, z-index is naively set to 1 for now
    pub fn grid_to_world_position(&self, position: Position) -> Vec3 {
        let y_offset = self.height as f32 / 2.0;
        let x_offset = self.width as f32 / 2.0;

        Vec3 {
            // NOTE: unit width is 1.0 so add 0.5 to center the position
            x: position.x as f32 - x_offset + 0.5,
            // The view of the model position is just an inversion along the y-axis.
            y: -position.y as f32 + y_offset - 0.5,
            z: 1.0,
        }
    }

    pub fn grid_to_tile_pos(&self, position: Position) -> TilePos {
        TilePos {
            x: position.x as u32,
            y: (self.height - position.y - 1) as u32,
        }
    }

    pub fn world_to_grid_position(&self, world_position: Vec2) -> Position {
        let x = world_position.x + (self.width() as f32 / 2.0) - 0.5;
        let y = -world_position.y + (self.height() as f32 / 2.0) - 0.5;
    
        Position {
            x: x.abs().round() as isize,
            y: y.abs().round() as isize,
        }
    }
}
