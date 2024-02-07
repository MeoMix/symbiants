use bevy::{prelude::*, sprite::Material2dPlugin};

use super::world_map::WorldMap;

#[derive(Default)]
pub struct WorldViewPlugin;

impl Plugin for WorldViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WorldMap>::default());
        // app.add_systems(
        //     Update,
        //     ().chain(), //TODO: Put in setup stuff here when implemented
        // )
    }
}