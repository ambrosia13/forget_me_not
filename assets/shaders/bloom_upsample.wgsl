#include assets/shaders/lib/header.wgsl
#include assets/shaders/lib/bloom.wgsl

@group(0) @binding(0)
var previous_upsample_mip_texture: texture_2d<f32>;

@group(0) @binding(1)
var upsample_sampler: sampler;

@group(0) @binding(2)
var downsample_texture: texture_2d<f32>;

@group(0) @binding(3)
var downsample_sampler: sampler;

@group(0) @binding(4)
var<uniform> lod_info: LodInfo;

@group(0) @binding(5)
var<uniform> screen: ScreenUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let prior = sample_tent(
        previous_upsample_mip_texture, 
        upsample_sampler, 
        in.uv, 
        vec2(screen.view.width >> lod_info.current_lod, screen.view.height >> lod_info.current_lod)
    );

    let current = textureSample(downsample_texture, downsample_sampler, in.uv);

    return prior + current;
}