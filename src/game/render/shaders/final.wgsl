struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
    inverse_view_projection_matrix: mat4x4<f32>,
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

    if color.x == 0.0 && color.y == 0.0 && color.z == 0.0 {
        let screen_space_pos = vec3(in.texcoord, 1.0);
        let clip_space_pos = screen_space_pos * 2.0 - 1.0;

        let temp = (camera.inverse_view_projection_matrix * vec4(clip_space_pos, 1.0));
        let view_space_pos = temp.xyz / temp.w;

        let view_dir = normalize(view_space_pos);

        color = vec4(view_dir, 1.0);
    }

    return color;
}