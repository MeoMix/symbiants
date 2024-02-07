#import bevy_sprite::mesh2d_bindings::mesh

struct MapData {
    world_size: vec2<u32>,
    tile_size: vec2<f32>
    atlas_size: vec2<f32>,

}

@group(1) @binding(0)
var<uniform> map: MapData;

@group(1) @binding(1)
var atlas_texture: texture2d<f32>;

#import bevy_sprite::mesh2d_bindings::{Vertex, VertexOutput}

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = map.tile_size * in.position;
    out.uv = in.uv * map.atlas_size;
    return out;
}

