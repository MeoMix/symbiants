pub mod dig;
pub mod emit_pheromone;
pub mod follow_pheromone;
pub mod set_pheromone_emitter;
pub mod travel;
pub mod wander;

use crate::common::ant::CraterOrientation;

use self::emit_pheromone::{LeavingFood, LeavingNest};
use bevy::prelude::*;

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    app_type_registry.write().register::<LeavingFood>();
    app_type_registry.write().register::<LeavingNest>();
    app_type_registry.write().register::<CraterOrientation>();
}
