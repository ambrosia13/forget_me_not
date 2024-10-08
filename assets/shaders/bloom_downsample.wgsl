#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/bloom.wgsl

@group(0) @binding(0)
var bloom_texture: texture_2d<f32>;

@group(0) @binding(1)
var bloom_sampler: sampler;

@group(0) @binding(2)
var<uniform> screen: ScreenUniforms;

@group(0) @binding(3)
var<uniform> lod_info: LodInfo;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return sample_13(
        bloom_texture, 
        bloom_sampler, 
        in.uv, 
        vec2(screen.view.width >> lod_info.current_lod, screen.view.height >> lod_info.current_lod)
    );
    // return textureSample(bloom_texture, bloom_sampler, in.uv);
}