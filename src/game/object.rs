use bevy_ecs::prelude::*;
use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::{
    render_state::RenderState,
    util::buffer::{AsGpuBytes, GpuBytes},
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    color: Vec3,
    emission: Vec3,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, color: Vec3, emission: Vec3) -> Self {
        Self {
            center,
            radius,
            color,
            emission,
        }
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn color(&self) -> Vec3 {
        self.color
    }

    pub fn emission(&self) -> Vec3 {
        self.emission
    }
}

impl AsGpuBytes for Sphere {
    fn as_gpu_bytes(&self) -> GpuBytes {
        let mut buf = GpuBytes::new();

        buf.write_vec3(self.center)
            .write_f32(self.radius)
            .write_vec3(self.color)
            .write_vec3(self.emission)
            .align();

        buf
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Plane {
    normal: Vec3,
    point: Vec3,
}

impl AsGpuBytes for Plane {
    fn as_gpu_bytes(&self) -> GpuBytes {
        let mut buf = GpuBytes::new();

        buf.write_vec3(self.normal).write_vec3(self.point).align();

        buf
    }
}

#[derive(Debug, Resource)]
pub struct Objects {
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
}

impl Objects {
    pub fn init(mut commands: Commands) {
        let mut objects = Objects {
            spheres: Vec::with_capacity(32),
            planes: Vec::with_capacity(32),
        };

        objects.push_sphere(Sphere::new(
            Vec3::new(0.1, 0.4, 0.3),
            0.2,
            Vec3::new(0.5, 0.6, 0.7),
            Vec3::new(0.9, 0.8, 1.0),
        ));

        commands.insert_resource(objects)
    }

    pub fn push_sphere(&mut self, sphere: Sphere) {
        self.spheres.insert(0, sphere);
    }
}

#[derive(Resource, Debug, Copy, Clone)]
pub struct ObjectsUniform {
    spheres: [Sphere; 32],
    planes: [Plane; 32],
    num_spheres: u32,
    num_planes: u32,
}

impl ObjectsUniform {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            num_spheres: 0,
            spheres: [Sphere::default(); 32],
            num_planes: 0,
            planes: [Plane::default(); 32],
        }
    }

    pub fn from_objects(objects: &Objects) -> Self {
        let mut spheres = [Sphere::default(); 32];
        let mut planes = [Plane::default(); 32];

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

impl AsGpuBytes for ObjectsUniform {
    fn as_gpu_bytes(&self) -> GpuBytes {
        let mut buf = GpuBytes::new();

        buf.write_u32(self.num_spheres);
        buf.write_u32(self.num_planes);

        for sphere in &self.spheres {
            buf.write_struct(sphere);
        }

        for plane in &self.planes {
            buf.write_struct(plane);
        }

        buf.align();

        buf
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
                    contents: bytemuck::cast_slice(objects_uniform.as_gpu_bytes().as_slice()),
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
            bytemuck::cast_slice(objects_uniform.as_gpu_bytes().as_slice()),
        );
    }
}
