use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};

#[derive(Resource)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection_matrix: Mat4,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection_matrix: Mat4::IDENTITY,
        }
    }
}
