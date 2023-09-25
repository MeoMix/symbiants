use bevy::prelude::*;
use bevy_save::SaveableRegistry;
use serde::{Deserialize, Serialize};

use crate::{common::register, grid::position::Position};

// TODO: This isn't great - prefer inferring this state at a local, ant level rather than relying on global flags to achieve behavior
#[derive(Resource, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Resource)]
pub struct Nest {
    position: Option<Position>,
    completed: bool,
}

impl Nest {
    pub fn is_started(&self) -> bool {
        self.position.is_some()
    }

    pub fn start(&mut self, position: Position) {
        self.position = Some(position);
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn position(&self) -> &Option<Position> {
        &self.position
    }
}

pub fn initialize_nest(
    app_type_registry: ResMut<AppTypeRegistry>,
    mut saveable_registry: ResMut<SaveableRegistry>,
    mut commands: Commands,
) {
    register::<Nest>(&app_type_registry, &mut saveable_registry);
    commands.init_resource::<Nest>();
}

pub fn deinitialize_nest(mut commands: Commands) {
    commands.remove_resource::<Nest>();
}
