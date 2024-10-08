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
    should_accumulate: u32,
}

// struct Camera {
//     view_projection_matrix: mat4x4<f32>,
//     view_matrix: mat4x4<f32>,
//     projection_matrix: mat4x4<f32>,

//     inverse_view_projection_matrix: mat4x4<f32>,
//     inverse_view_matrix: mat4x4<f32>,
//     inverse_projection_matrix: mat4x4<f32>,

//     previous_view_projection_matrix: mat4x4<f32>,
//     previous_view_matrix: mat4x4<f32>,
//     previous_projection_matrix: mat4x4<f32>,

//     position: vec3<f32>,
//     previous_position: vec3<f32>,
// }

// struct View {
//     width: u32,
//     height: u32,
//     frame_count: u32,
// }
