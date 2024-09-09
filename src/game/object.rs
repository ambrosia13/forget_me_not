use bevy_ecs::prelude::*;
use bytemuck::Zeroable;
use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::render_state::RenderState;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pub center: Vec3,
    _padding: u32,
    pub color: Vec3,
    _padding_1: u32,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, color: Vec3) -> Self {
        Self {
            center,
            _padding: 0,
            color,
            _padding_1: 1,
            radius,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Plane {
    bleh: u32,
    bleh_2: u32,
    bleh_3: u32,
    bleh_4: u32,
}

#[derive(Debug, Resource)]
pub struct Objects {
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
}

impl Objects {
    pub fn init(mut commands: Commands) {
        commands.insert_resource(Objects {
            spheres: Vec::with_capacity(32),
            planes: Vec::with_capacity(32),
        })
    }

    pub fn push_sphere(&mut self, sphere: Sphere) {
        self.spheres.insert(0, sphere);
    }
}

#[repr(C)]
#[derive(Resource, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectsUniform {
    num_spheres: u32,
    num_planes: u32,
    spheres: [Sphere; 32],
    planes: [Plane; 32],
}

impl ObjectsUniform {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            num_spheres: 0,
            spheres: [Sphere::zeroed(); 32],
            num_planes: 0,
            planes: [Plane::zeroed(); 32],
        }
    }

    pub fn from_objects(objects: &Objects) -> Self {
        let mut spheres = [Sphere::zeroed(); 32];
        let mut planes = [Plane::zeroed(); 32];

        for (i, &sphere) in objects.spheres.iter().enumerate() {
            spheres[i] = sphere;
        }
        for (i, &plane) in objects.planes.iter().enumerate() {
            planes[i] = plane;
        }

        Self {
            num_spheres: objects.spheres.len() as u32,
            spheres,
            num_planes: objects.planes.len() as u32,
            planes,
        }
    }

    pub fn init(mut commands: Commands) {
        commands.insert_resource(ObjectsUniform::new());
    }

    pub fn update(mut objects_uniform: ResMut<ObjectsUniform>, objects: Res<Objects>) {
        *objects_uniform = ObjectsUniform::from_objects(&objects);
    }
}

#[derive(Resource)]
pub struct ObjectsBuffer {
    pub buffer: wgpu::Buffer,
}

impl ObjectsBuffer {
    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        objects_uniform: Res<ObjectsUniform>,
    ) {
        commands.insert_resource(ObjectsBuffer {
            buffer: render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Objects Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[*objects_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
        })
    }

    pub fn update(
        objects_uniform: Res<ObjectsUniform>,
        objects_buffer: Res<ObjectsBuffer>,
        render_state: Res<RenderState>,
    ) {
        render_state.queue.write_buffer(
            &objects_buffer.buffer,
            0,
            bytemuck::cast_slice(&[*objects_uniform]),
        );
    }
}
