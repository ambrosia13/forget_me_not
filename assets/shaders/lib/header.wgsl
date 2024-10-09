struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
}

struct Camera {
    view_projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,

    inverse_view_projection_matrix: mat4x4<f32>,
    inverse_view_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,

    previous_view_projection_matrix: mat4x4<f32>,
    previous_view_matrix: mat4x4<f32>,
    previous_projection_matrix: mat4x4<f32>,

    position: vec3<f32>,
    previous_position: vec3<f32>,

    view: vec3<f32>,
    previous_view: vec3<f32>,
}

struct View {
    width: u32,
    height: u32,

    frame_count: u32,
}

struct ScreenUniforms {
    camera: Camera,
    view: View,
}

fn get_taa_offset(frame: u32) -> vec2<f32> {
    var taa_offsets = array<vec2<f32>, 8>(
        vec2( 0.125,-0.375),
        vec2(-0.125, 0.375),
        vec2( 0.625, 0.125),
        vec2( 0.375,-0.625),
        vec2(-0.625, 0.625),
        vec2(-0.875,-0.125),
        vec2( 0.375,-0.875),
        vec2( 0.875, 0.875)
    );

    return taa_offsets[frame % 8];
}