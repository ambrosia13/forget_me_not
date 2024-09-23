struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
    inverse_view_projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    inverse_view_matrix: mat4x4<f32>,
    previous_view_projection_matrix: mat4x4<f32>,
    pos: vec3<f32>,
    previous_pos: vec3<f32>,
    view_width: u32,
    view_height: u32,
    frame_count: u32,
}

@group(0) @binding(0)
var bloom_texture: texture_2d<f32>;

@group(0) @binding(1)
var bloom_sampler: sampler;

@group(0) @binding(2)
var<uniform> camera: CameraUniform;

@group(0) @binding(3)
var<uniform> lod: u32;

fn sample_tent(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>, resolution: vec2<u32>) -> vec4<f32> {
    let dist = 1.0 / vec2<f32>(resolution);
    let d = vec4(1.0, 1.0, -1.0, 0.0) * dist.xyxy;

    var sum = vec4(0.0);

    sum += textureSample(bloom_texture, bloom_sampler, uv - d.xy);
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.wy) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv - d.zy);
    sum += textureSample(bloom_texture, bloom_sampler, uv + d.zw) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv) * 4.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv + d.xw) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv + d.zy);
    sum += textureSample(bloom_texture, bloom_sampler, uv + d.wy) * 2.0;
    sum += textureSample(bloom_texture, bloom_sampler, uv + d.xy);

    return sum * (1.0 / 16.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return sample_tent(bloom_texture, bloom_sampler, in.uv, vec2(camera.view_width >> lod, camera.view_height >> lod));
    // return textureSample(bloom_texture, bloom_sampler, in.uv);
}