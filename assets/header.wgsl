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
