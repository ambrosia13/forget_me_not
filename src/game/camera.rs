use crate::game::input::{KeyboardInput, MouseMotion};
use crate::render_state::{LastFrameInstant, RenderState, WindowResizeEvent};
use bevy_ecs::prelude::*;
use glam::{Mat3, Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::keyboard::KeyCode;

pub fn init(mut commands: Commands, render_state: Res<RenderState>) {
    let mut camera = Camera::new(
        Vec3::new(2.0, 0.0, 5.0),
        Quat::from_rotation_y(0.0),
        45.0,
        render_state.size,
        0.01,
        100.0,
    );

    camera.look_at(Vec3::ZERO);

    commands.insert_resource(camera);
}

pub fn update(
    mut camera: ResMut<Camera>,
    mut resize_events: EventReader<WindowResizeEvent>,
    mouse_motion: Res<MouseMotion>,
    keyboard_input: Res<KeyboardInput>,
    last_frame_instant: Res<LastFrameInstant>,
) {
    for event in resize_events.read() {
        camera.reconfigure_aspect(event.0);
    }

    camera.update_rotation(
        mouse_motion.delta_x as f32,
        mouse_motion.delta_y as f32,
        0.25,
    );

    let delta_time = last_frame_instant.elapsed().as_secs_f32();

    let mut velocity = Vec3::ZERO;
    let forward = camera.forward_xz();
    let right = camera.right_xz();
    let up = Vec3::Y;

    if keyboard_input.pressed(KeyCode::KeyW) {
        velocity += forward;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        velocity -= forward;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        velocity += right;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        velocity -= right;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        velocity += up;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        velocity -= up;
    }

    velocity = velocity.normalize_or_zero();
    let movement_speed = 0.9 * delta_time;
    camera.position += velocity * movement_speed;
}

pub fn init_uniform_buffer(mut commands: Commands, render_state: Res<RenderState>) {
    let camera_uniform = CameraUniform::new();

    commands.insert_resource(CameraBuffer {
        buffer: render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
    });

    commands.insert_resource(camera_uniform);
}

pub fn update_uniform_buffer(
    mut camera_uniform: ResMut<CameraUniform>,
    camera_buffer: Res<CameraBuffer>,
    camera: Res<Camera>,
    render_state: Res<RenderState>,
) {
    camera_uniform.update(&camera);

    render_state.queue.write_buffer(
        &camera_buffer.buffer,
        0,
        bytemuck::cast_slice(&[*camera_uniform]),
    );
}

#[derive(Resource)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pitch: f32,
    yaw: f32,
    pub fov: f32,
    aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        fov: f32,
        window_size: PhysicalSize<u32>,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            rotation,
            pitch: 0.0,
            yaw: 0.0,
            fov,
            aspect: window_size.width as f32 / window_size.height as f32,
            near,
            far,
        }
    }

    pub fn reconfigure_aspect(&mut self, window_size: PhysicalSize<u32>) {
        self.aspect = window_size.width as f32 / window_size.height as f32;
    }

    pub fn look_at(&mut self, target: Vec3) {
        let forward = (target - self.position).normalize();
        let right = Vec3::Y.cross(forward).normalize();
        let up = forward.cross(right);

        let look_rotation_matrix = Mat3::from_cols(right, up, forward);
        self.rotation = Quat::from_mat3(&look_rotation_matrix);
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn forward_xz(&self) -> Vec3 {
        let forward = self.forward();
        Vec3::new(forward.x, 0.0, forward.z).normalize()
    }

    pub fn right(&self) -> Vec3 {
        -(self.rotation * Vec3::X)
    }

    pub fn right_xz(&self) -> Vec3 {
        let right = self.right();
        Vec3::new(right.x, 0.0, right.z).normalize()
    }

    const TEST: Mat4 = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
    ]);

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward(), Vec3::Y)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    pub fn update_rotation(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        let yaw_delta = -delta_x * sensitivity;
        let pitch_delta = delta_y * sensitivity;

        self.yaw += yaw_delta;

        self.pitch += pitch_delta;
        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let yaw_quat = Quat::from_rotation_y(self.yaw.to_radians());
        let pitch_quat = Quat::from_rotation_x(self.pitch.to_radians());

        self.rotation = yaw_quat * pitch_quat;
    }
}

#[repr(C)]
#[derive(Resource, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection_matrix: Mat4,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection_matrix: Mat4::IDENTITY,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.get_projection_matrix() * camera.get_view_matrix();
    }
}

#[derive(Resource)]
pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
}
