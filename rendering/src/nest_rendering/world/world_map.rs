use bevy::{
    math::vec2, 
    ecs::reflect, prelude::*, render::render_resource::{
            AsBindGroup, ShaderRef
        }, sprite::Material2d
};

#[derive(Component, Asset, Default, Debug, Clone, Reflect, AsBindGroup)]
#[reflect(Component)]
pub struct WorldMap {
    pub world_size: UVec2,
    pub tile_size: Vec2,
    pub atlas_size: Vec2,

}

impl Material2d for WorldMap {
    fn fragment_shader() -> ShaderRef {
        "shaders/world.wgsl".into()
    }
}