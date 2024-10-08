use bevy_ecs::prelude::*;
use glam::Vec3;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::{
    render_state::RenderState,
    util::buffer::{AsStd140Bytes, Std140Bytes},
};

use super::material::{Material, MaterialType};

const PAD_THICKNESS: f32 = 0.00025;

#[derive(Default, Debug, Clone, Copy)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn pad(self) -> Self {
        Self {
            radius: self.radius - PAD_THICKNESS,
            ..self
        }
    }
}

impl AsStd140Bytes for Sphere {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_vec3(self.center)
            .write_f32(self.radius)
            .write_struct(&self.material)
            .align();

        buf
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Plane {
    normal: Vec3,
    point: Vec3,
    material: Material,
}

impl Plane {
    pub fn new(normal: Vec3, point: Vec3, material: Material) -> Self {
        Self {
            normal,
            point,
            material,
        }
    }
}

impl AsStd140Bytes for Plane {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_vec3(self.normal)
            .write_vec3(self.point)
            .write_struct(&self.material)
            .align();

        buf
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
    material: Material,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Self { min, max, material }
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn pad(self) -> Self {
        Self {
            min: self.min + PAD_THICKNESS,
            max: self.max - PAD_THICKNESS,
            ..self
        }
    }
}

impl AsStd140Bytes for Aabb {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_vec3(self.min)
            .write_vec3(self.max)
            .write_struct(&self.material)
            .align();

        buf
    }
}

#[derive(Debug, Resource)]
pub struct Objects {
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
    pub aabbs: Vec<Aabb>,
}

impl Objects {
    pub fn init(mut commands: Commands) {
        let mut objects = Objects {
            spheres: Vec::with_capacity(32),
            planes: Vec::with_capacity(32),
            aabbs: Vec::with_capacity(32),
        };

        objects.random_scene();

        commands.insert_resource(objects)
    }

    pub fn random_scene(&mut self) {
        self.spheres.clear();
        self.planes.clear();
        self.aabbs.clear();

        self.planes.push(Plane::new(
            Vec3::Y,
            Vec3::ZERO,
            Material {
                ty: MaterialType::Lambertian,
                albedo: Vec3::ONE,
                emission: Vec3::ZERO,
                roughness: 0.0,
                ior: 0.0,
            },
        ));

        let region_size = 5;
        let regions_radius = 2;

        for x in -regions_radius..=regions_radius {
            for z in -regions_radius..=regions_radius {
                let x = (x * region_size) as f32;
                let z = (z * region_size) as f32;

                let max_offset = region_size as f32 / 2.0 * 0.8;
                let min_radius = region_size as f32 / 2.0 * 0.2;

                let offset = rand::thread_rng().gen_range(-max_offset..=max_offset);

                let rand_radius = || {
                    rand::thread_rng()
                        .gen_range(min_radius..=(max_offset - offset.abs() + min_radius))
                        .sqrt()
                };

                match rand::thread_rng().gen_range(0..2) {
                    0 => {
                        let radius = rand_radius();

                        self.push_sphere(Sphere::new(
                            Vec3::new(x + offset, radius, z + offset),
                            radius,
                            Material::random(),
                        ))
                    }
                    1 => {
                        let radius_x = rand_radius();
                        let radius_y = rand_radius();
                        let radius_z = rand_radius();

                        self.push_aabb(Aabb::new(
                            Vec3::new(x + offset - radius_x, 0.0, z + offset - radius_z),
                            Vec3::new(x + offset + radius_x, 2.0 * radius_y, z + offset + radius_z),
                            Material::random(),
                        ))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn push_sphere(&mut self, sphere: Sphere) {
        self.spheres.insert(0, sphere);
    }

    pub fn push_plane(&mut self, plane: Plane) {
        self.planes.insert(0, plane);
    }

    pub fn push_aabb(&mut self, aabb: Aabb) {
        self.aabbs.insert(0, aabb);
    }

    pub fn inside_sphere(&self, pos: Vec3) -> Option<Sphere> {
        let mut inside_sphere: Option<Sphere> = None;

        for &sphere in &self.spheres {
            if pos.distance(sphere.center()) < sphere.radius() {
                inside_sphere = match inside_sphere {
                    // select the smallest sphere
                    Some(sph) if sph.radius() < sphere.radius() => Some(sph),
                    Some(_) => Some(sphere),
                    None => None,
                };
            }
        }

        inside_sphere
    }

    pub fn inside_aabb(&self, pos: Vec3) -> Option<Aabb> {
        fn radius(aabb: Aabb) -> f32 {
            (aabb.max() - aabb.min()).length()
        }

        let mut inside_aabb: Option<Aabb> = None;

        for &aabb in &self.aabbs {
            if pos.cmpgt(aabb.min()).all() && pos.cmplt(aabb.max()).all() {
                inside_aabb = match inside_aabb {
                    Some(previous_aabb) if radius(previous_aabb) < radius(aabb) => {
                        Some(previous_aabb)
                    }
                    Some(_) => Some(aabb),
                    None => None,
                };
            }
        }

        inside_aabb
    }
}

#[derive(Resource, Debug, Copy, Clone)]
pub struct ObjectsUniform {
    num_spheres: u32,
    num_planes: u32,
    num_aabbs: u32,
    spheres: [Sphere; 32],
    planes: [Plane; 32],
    aabbs: [Aabb; 32],
}

impl ObjectsUniform {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            num_spheres: 0,
            num_planes: 0,
            num_aabbs: 0,
            spheres: [Sphere::default(); 32],
            planes: [Plane::default(); 32],
            aabbs: [Aabb::default(); 32],
        }
    }

    pub fn from_objects(objects: &Objects) -> Self {
        let mut spheres = [Sphere::default(); 32];
        let mut planes = [Plane::default(); 32];
        let mut aabbs = [Aabb::default(); 32];

        for (i, &sphere) in objects.spheres.iter().enumerate().take(32) {
            spheres[i] = sphere.pad();
        }
        for (i, &plane) in objects.planes.iter().enumerate().take(32) {
            planes[i] = plane;
        }
        for (i, &aabb) in objects.aabbs.iter().enumerate().take(32) {
            aabbs[i] = aabb.pad();
        }

        Self {
            num_spheres: objects.spheres.len() as u32,
            num_planes: objects.planes.len() as u32,
            num_aabbs: objects.aabbs.len() as u32,
            spheres,
            planes,
            aabbs,
        }
    }

    pub fn init(mut commands: Commands) {
        commands.insert_resource(ObjectsUniform::new());
    }

    pub fn update(mut objects_uniform: ResMut<ObjectsUniform>, objects: Res<Objects>) {
        *objects_uniform = ObjectsUniform::from_objects(&objects);
    }
}

impl AsStd140Bytes for ObjectsUniform {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_u32(self.num_spheres);
        buf.write_u32(self.num_planes);
        buf.write_u32(self.num_aabbs);

        for sphere in &self.spheres {
            buf.write_struct(sphere);
        }

        for plane in &self.planes {
            buf.write_struct(plane);
        }

        for aabb in &self.aabbs {
            buf.write_struct(aabb);
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
                    contents: bytemuck::cast_slice(objects_uniform.as_std140().as_slice()),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }),
        })
    }

    pub fn update(
        objects: Res<Objects>,
        objects_uniform: Res<ObjectsUniform>,
        objects_buffer: Res<ObjectsBuffer>,
        render_state: Res<RenderState>,
    ) {
        // we don't need to write to the buffer unless an object was added or removed
        if objects.is_changed() {
            render_state.queue.write_buffer(
                &objects_buffer.buffer,
                0,
                bytemuck::cast_slice(objects_uniform.as_std140().as_slice()),
            );
        }
    }
}
