use crate::game::input::{KeyboardInput, MouseMotion};
use crate::render_state::{LastFrameInstant, RenderState, WindowResizeEvent};
use crate::util::buffer::{AsStd140Bytes, Std140Bytes};
use bevy_ecs::prelude::*;
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::keyboard::KeyCode;

#[derive(Resource, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pitch: f32,
    yaw: f32,
    pub fov: f32,
    aspect: f32,
    pub near: f32,
    pub far: f32,

    view_width: u32,
    view_height: u32,

    previous_view_direction: Vec3,
}

impl Camera {
    pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
        Vec4::new(1.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, -1.0, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Vec3,
        rotation: Quat,
        fov: f32,
        window_size: PhysicalSize<u32>,
        near: f32,
        far: f32,
        view_width: u32,
        view_height: u32,
    ) -> Self {
        Self {
            position,
            rotation,
            pitch: 0.0,
            yaw: -90.0,
            fov,
            aspect: window_size.width as f32 / window_size.height as f32,
            near,
            far,
            view_width,
            view_height,
            previous_view_direction: Vec3::ZERO,
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

        (self.yaw, self.pitch) = self.yaw_pitch_from_rotation();
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

        let yaw_quat = self.yaw_quat();
        let pitch_quat = self.pitch_quat();

        self.rotation = yaw_quat * pitch_quat;
    }

    pub fn yaw_quat(&self) -> Quat {
        Quat::from_rotation_y(self.yaw.to_radians())
    }

    pub fn pitch_quat(&self) -> Quat {
        Quat::from_rotation_x(self.pitch.to_radians())
    }

    pub fn yaw_pitch_from_rotation(&self) -> (f32, f32) {
        let forward = self.rotation * Vec3::Z;

        let yaw = forward.z.atan2(forward.x).to_degrees();
        let pitch = forward.y.asin().to_degrees();

        (yaw, pitch)
    }

    pub fn init(mut commands: Commands, render_state: Res<RenderState>) {
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::from_rotation_y(0.0),
            45.0,
            render_state.size,
            0.01,
            100.0,
            render_state.size.width,
            render_state.size.height,
        );

        camera.look_at(Vec3::new(0.0, 0.0, -1.0));

        commands.insert_resource(camera);
    }

    pub fn update(
        mut camera: ResMut<Camera>,
        mut resize_events: EventReader<WindowResizeEvent>,
        mouse_motion: Option<Res<MouseMotion>>,
        keyboard_input: Res<KeyboardInput>,
        last_frame_instant: Res<LastFrameInstant>,
    ) {
        camera.previous_view_direction = camera.forward();

        for event in resize_events.read() {
            camera.reconfigure_aspect(event.0);
        }

        if let Some(mouse_motion) = mouse_motion {
            camera.update_rotation(
                mouse_motion.delta_x as f32,
                mouse_motion.delta_y as f32,
                0.25,
            );
        }

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
        let movement_speed = 50.0 * delta_time;
        camera.position += velocity * movement_speed;
    }
}

#[derive(Resource, Default, Debug)]
pub struct CameraUniform {
    // view_projection_matrix: Mat4,
    // inverse_view_projection_matrix: Mat4,
    // view_matrix: Mat4,
    // inverse_view_matrix: Mat4,
    // previous_view_projection_matrix: Mat4,
    // pos: Vec3,
    // previous_pos: Vec3,
    // view_width: u32,
    // view_height: u32,
    // frame_count: u32,
    // should_accumulate: u32,
    view_projection_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,

    inverse_view_projection_matrix: Mat4,
    inverse_view_matrix: Mat4,
    inverse_projection_matrix: Mat4,

    previous_view_projection_matrix: Mat4,
    previous_view_matrix: Mat4,
    previous_projection_matrix: Mat4,

    position: Vec3,
    previous_position: Vec3,

    view: Vec3,
    previous_view: Vec3,
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera) -> Self {
        let view_matrix = camera.get_view_matrix();
        let projection_matrix = camera.get_projection_matrix();
        let view_projection_matrix = projection_matrix * view_matrix;

        let inverse_view_matrix = view_matrix.inverse();
        let inverse_projection_matrix = projection_matrix.inverse();
        let inverse_view_projection_matrix = view_projection_matrix.inverse();

        let position = camera.position;
        let view = camera.forward();

        Self {
            view_projection_matrix,
            view_matrix,
            projection_matrix,

            inverse_view_projection_matrix,
            inverse_view_matrix,
            inverse_projection_matrix,

            previous_view_projection_matrix: view_projection_matrix,
            previous_view_matrix: view_matrix,
            previous_projection_matrix: projection_matrix,

            position,
            previous_position: position,

            view,
            previous_view: view,
        }
    }

    pub fn init(mut commands: Commands) {
        commands.insert_resource(CameraUniform::default());
    }

    pub fn update(mut camera_uniform: ResMut<CameraUniform>, camera: Res<Camera>) {
        // Values of the previous uniform become the "previous_" values of the current uniform
        let previous_view_projection_matrix = camera_uniform.view_projection_matrix;
        let previous_view_matrix = camera_uniform.view_matrix;
        let previous_projection_matrix = camera_uniform.projection_matrix;

        let previous_position = camera_uniform.position;
        let previous_view = camera_uniform.view;

        *camera_uniform = CameraUniform {
            previous_view_projection_matrix,
            previous_view_matrix,
            previous_projection_matrix,

            previous_position,
            previous_view,

            ..CameraUniform::from_camera(&camera)
        };
    }
}

impl AsStd140Bytes for CameraUniform {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_mat4(self.view_projection_matrix)
            .write_mat4(self.view_matrix)
            .write_mat4(self.projection_matrix)
            .write_mat4(self.inverse_view_projection_matrix)
            .write_mat4(self.inverse_view_matrix)
            .write_mat4(self.inverse_projection_matrix)
            .write_mat4(self.previous_view_projection_matrix)
            .write_mat4(self.previous_view_matrix)
            .write_mat4(self.previous_projection_matrix)
            .write_vec3(self.position)
            .write_vec3(self.previous_position)
            .write_vec3(self.view)
            .write_vec3(self.previous_view)
            .align();

        buf
    }
}

#[derive(Resource, Default)]
pub struct ViewUniform {
    width: u32,
    height: u32,

    frame_count: u32,
}

impl ViewUniform {
    pub fn new(render_state: &RenderState, frame_count: u32) -> Self {
        Self {
            width: render_state.size.width,
            height: render_state.size.height,
            frame_count,
        }
    }

    pub fn init(world: &mut World) {
        world.insert_resource(ViewUniform::default());
    }

    pub fn update(mut view_uniform: ResMut<ViewUniform>, render_state: Res<RenderState>) {
        let frame_count = view_uniform.frame_count.wrapping_add(1);
        *view_uniform = ViewUniform::new(&render_state, frame_count);
    }
}

impl AsStd140Bytes for ViewUniform {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_u32(self.width)
            .write_u32(self.height)
            .write_u32(self.frame_count)
            .align();

        buf
    }
}

#[derive(Resource)]
pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
}

impl CameraBuffer {
    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        camera_uniform: Res<CameraUniform>,
        view_uniform: Res<ViewUniform>,
    ) {
        let mut buffer_data = Std140Bytes::new();

        buffer_data.write_struct(&*camera_uniform);
        buffer_data.write_struct(&*view_uniform);
        buffer_data.align();

        commands.insert_resource(CameraBuffer {
            buffer: render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Uniform Buffer"),
                    contents: buffer_data.as_slice(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
        });
    }

    pub fn update(
        camera_uniform: Res<CameraUniform>,
        view_uniform: Res<ViewUniform>,
        camera_buffer: Res<CameraBuffer>,
        render_state: Res<RenderState>,
    ) {
        let mut buffer_data = Std140Bytes::new();

        buffer_data.write_struct(&*camera_uniform);
        buffer_data.write_struct(&*view_uniform);
        buffer_data.align();

        render_state
            .queue
            .write_buffer(&camera_buffer.buffer, 0, buffer_data.as_slice());
    }
}
