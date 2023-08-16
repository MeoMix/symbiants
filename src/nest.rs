use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: This isn't great - prefer inferring this state at a local, ant level rather than relying on global flags to achieve behavior
#[derive(Resource, Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Resource)]
pub struct Nest {
    started: bool,
    completed: bool,
}

impl Nest {
    pub fn is_started(&self) -> &bool {
        &self.started
    }

    pub fn start(&mut self) {
        self.started = true;
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
}
