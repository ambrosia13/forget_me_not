struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

@group(0) @binding(0)
var previous_upsample_mip_texture: texture_2d<f32>;

@group(0) @binding(1)
var upsample_sampler: sampler;

@group(0) @binding(2)
var downsample_texture: texture_2d<f32>;

@group(0) @binding(3)
var downsample_sampler: sampler;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let prior = textureSample(previous_upsample_mip_texture, upsample_sampler, in.uv);
    let current = textureSample(downsample_texture, downsample_sampler, in.uv);

    return prior + current;
}