use glam::Vec3;
use rand::Rng;

use crate::util::buffer::{AsStd140Bytes, Std140Bytes};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum MaterialType {
    #[default]
    Lambertian = 0,
    Metal = 1,
    Dielectric = 2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Material {
    pub ty: MaterialType,
    pub albedo: Vec3,
    pub emission: Vec3,
    pub roughness: f32,
    pub ior: f32,
}

impl Material {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            ty: match rng.gen_range(0..3) {
                0 => MaterialType::Lambertian,
                1 => MaterialType::Metal,
                2 => MaterialType::Dielectric,
                _ => unreachable!(),
            },
            albedo: Vec3::new(
                rng.gen::<f32>().powf(2.2),
                rng.gen::<f32>().powf(2.2),
                rng.gen::<f32>().powf(2.2),
            ),
            emission: match rng.gen_bool(0.1) {
                // less emission is more common
                true => Vec3::new(
                    rng.gen_range(1.0f32..10.0),
                    rng.gen_range(1.0f32..10.0),
                    rng.gen_range(1.0f32..10.0),
                ),
                false => Vec3::ZERO,
            },
            roughness: rng.gen_range(0.0f32..1.0).powi(3),
            ior: rng.gen_range(0.5f32..3.0f32).powf(0.5),
        }
    }
}

impl AsStd140Bytes for Material {
    fn as_std140(&self) -> Std140Bytes {
        let mut buf = Std140Bytes::new();

        buf.write_u32(self.ty as u32)
            .write_vec3(self.albedo)
            .write_vec3(self.emission)
            .write_f32(self.roughness)
            .write_f32(self.ior)
            .align();

        buf
    }
}
