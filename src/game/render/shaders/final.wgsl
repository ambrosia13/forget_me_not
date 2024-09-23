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
var color_texture: texture_2d<f32>;
@group(0) @binding(1)
var color_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(color_texture, color_sampler, in.uv);

    let screen_space_pos = vec3(in.texcoord, 1.0);
    let clip_space_pos = screen_space_pos * 2.0 - 1.0;

    let temp = (camera.inverse_view_projection_matrix * vec4(clip_space_pos, 1.0));
    let world_space_pos = temp.xyz / temp.w;
    let view_space_pos = world_space_pos - camera.pos;

    let view_dir = normalize(view_space_pos);

    // if depth == 1.0 {
    //     let mix_factor = smoothstep(0.0, 0.2, clamp(view_dir.y, 0.0, 1.0));
    //     color = vec4(mix(vec3(1.0, 1.0, 1.0), vec3(0.25, 0.5, 1.0), mix_factor), 1.0);
    // }

    color = vec4(pow(color.rgb, vec3(1.0 / 2.2)), color.a);

    return color;
}