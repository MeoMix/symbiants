pub mod world_map;
pub use self::world_map::{initialize_world_assets, WorldMap};

use bevy::{prelude::*, sprite::Material2dPlugin};

#[derive(Default)]
pub struct WorldViewPlugin;

impl Plugin for WorldViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WorldMap>::default());
        app.add_systems(Update, initialize_world_assets);
    }
}
