#include assets/shaders/header.wgsl
#include assets/shaders/bloom_header.wgsl

@group(0) @binding(0)
var downsample_texture: texture_2d<f32>;

@group(0) @binding(1)
var downsample_sampler: sampler;

@group(0) @binding(2)
var<uniform> lod_info: LodInfo;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(downsample_texture, downsample_sampler, in.uv);
}