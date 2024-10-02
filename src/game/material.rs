use glam::Vec3;

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
