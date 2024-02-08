use std::panic::AssertUnwindSafe;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(Component, Asset, Default, Debug, Clone, Reflect, AsBindGroup)]
#[reflect(Component)]
pub struct WorldMap {
    pub world_size: UVec2,
    pub tile_size: Vec2,
    pub atlas_size: Vec2,
    pub tile_atlas: Handle<TextureAtlas>,
}

impl Material2d for WorldMap {
    fn fragment_shader() -> ShaderRef {
        "shaders/world.wgsl".into()
    }
}

impl WorldMap {
    pub fn new(
        world_size: UVec2,
        tile_size: Vec2,
        atlas_size: Vec2,
        tile_atlas: Handle<TextureAtlas>,
    ) -> Self {
        Self {
            world_size,
            tile_size,
            atlas_size,
            tile_atlas,
        }
    }
}

pub fn initialize_world_assets(
    map: ResMut<Assets<WorldMap>>,
    map_handles: Query<&Handle<WorldMap>>,
    mut images: ResMut<Assets<Image>>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
) {
    for ev in ev_asset.iter() {
        for handle in map_handles.iter() {
            let Some(map) = map.get(handle) else {
                /* World has no texture atlas */
                continue;
            };
        }
    }
}
