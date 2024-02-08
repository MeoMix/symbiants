use bevy::log::debug;

use bevy::render;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::Material2d,
};

#[derive(ShaderType, Clone, Debug, Reflect, AsBindGroup)]
pub struct MapDataUniform {
    pub map_size: UVec2,
    pub tile_size: Vec2,
    pub atlas_size: Vec2,
    pub transform_local: Mat3,
    pub transform_world: Mat3,
    pub size_world: Vec2,
}

impl Default for MapDataUniform {
    fn default() -> Self {
        Self {
            map_size: default(),
            tile_size: default(),
            atlas_size: default(),
            transform_local: Mat3::IDENTITY,
            transform_world: default(),
            size_world: default(),
        }
    }
}

#[derive(Component, Asset, Default, Debug, Clone, Reflect, AsBindGroup)]
#[reflect(Component)]
pub struct WorldMap {
    #[uniform(0)]
    pub map_data: MapDataUniform,

    #[storage(32)]
    pub tile_map: Vec<u32>,

    #[texture(33)]
    #[sampler(34)]
    pub atlas: Handle<Image>,
}

impl Material2d for WorldMap {
    fn fragment_shader() -> ShaderRef {
        "shaders/world.wgsl".into()
    }
}

impl WorldMap {
    pub fn update(&mut self, images: &Assets<Image>) -> bool {
        let atlas = match images.get(&self.atlas) {
            Some(atlas) => atlas,
            None => return false,
        };

        self.map_data.atlas_size = atlas.size().as_vec2();
        true
    }
}

// TODO: This probably needs to stay here, while the rest of world handling
// can be moved to the parent module so it can be re-used for crater rendering.
pub fn create_new_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<WorldMap>>,
) {
    let world_map = WorldMap {
        map_data: MapDataUniform {
            tile_size: Vec2::new(32.0, 32.0),
            ..Default::default()
        },
        tile_map: vec![0; 100],
        atlas: asset_server.load("textures/element/sprite_sheet.png"),
    };

    materials.add(world_map);
}

pub fn initialize_tile_worlds(
    mut assets: ResMut<Assets<WorldMap>>,
    worlds: Query<(Entity, &Handle<WorldMap>)>,
    images: Res<Assets<Image>>,
) {
    debug!(target: "world", "Initializing tile worlds");
    for (entity, handle) in worlds.iter() {
        let Some(map) = assets.get_mut(handle) else {
            warn!(target: "world", "WorldMap not loaded yet: {:?}", handle);
            continue;
            // Not loaded yet, probably. Need to handle.
        };

        map.update(images.as_ref());
        warn!(target: "world", "WorldMap updated: {:?}", map);
    }
}
