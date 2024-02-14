pub mod birthing;
pub mod chambering;
pub mod dig;
pub mod drop;
pub mod nest_expansion;
pub mod nesting;
pub mod sleep;
pub mod travel;
pub mod tunneling;
pub mod walk;

use self::{birthing::Birthing, chambering::Chambering, sleep::Asleep, tunneling::Tunneling};
use bevy::prelude::*;

pub fn register_ant(app_type_registry: ResMut<AppTypeRegistry>) {
    // TODO: This might be nest-specific, but maybe needs to be supported at crater just in case
    app_type_registry.write().register::<Asleep>();

    // TODO: These seem nest-specific
    app_type_registry.write().register::<Birthing>();
    app_type_registry.write().register::<Tunneling>();
    app_type_registry.write().register::<Chambering>();
}

// TODO: tests
