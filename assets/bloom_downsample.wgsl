struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

@group(0) @binding(0)
var bloom_texture: texture_2d<f32>;

@group(0) @binding(1)
var bloom_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(bloom_texture, bloom_sampler, in.uv);
}