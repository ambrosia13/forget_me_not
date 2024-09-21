struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

@group(0) @binding(0)
var color_texture: texture_2d<f32>;

@group(0) @binding(1)
var color_sampler: sampler;

@group(0) @binding(2)
var upsample_texture: texture_2d<f32>;

@group(0) @binding(3)
var upsample_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(color_texture, color_sampler, in.uv);
    let bloom = textureSample(upsample_texture, upsample_sampler, in.uv) / 7.0;

    return mix(color, bloom, 0.2);
}