struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

@group(0) @binding(0)
var bloom_texture: texture_2d<f32>;

@group(0) @binding(1)
var bloom_sampler: sampler;

fn sample_tent(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, resolution: vec2<u32>) -> vec4<f32> {
    let dist = 1.0 / vec2<f32>(resolution);
    let d = vec4(1.0, 1.0, -1.0, 0.0) * dist.xyxy;

    var sum = vec4(0.0);

    sum += textureSample(bloom_texture, bloom_sampler, uv - d.xy);
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.wy) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.zy);
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.zw) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv) * 4.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.xw) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.zy);
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.wy) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.xy);

    return sum * (1.0 / 16.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // if in.texcoord.x > 0.5 {
    //     return sample_tent(bloom_texture, bloom_sampler, in.uv, vec2(1920u, 1080u) / 64);
    // } else {
    return textureSample(bloom_texture, bloom_sampler, in.uv);
    // }
}