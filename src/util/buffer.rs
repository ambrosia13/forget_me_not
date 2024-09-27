use glam::{IVec2, IVec3, IVec4, Mat3, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

pub trait AsStd140Bytes {
    fn as_std140(&self) -> Std140Bytes;
}

#[derive(Default)]
pub struct Std140Bytes {
    bytes: Vec<u8>,
    alignment: usize,
}

impl Std140Bytes {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            alignment: 0,
        }
    }

    fn write_slice(&mut self, data: &[u8], align: usize) {
        self.alignment = self.alignment.max(align);

        let offset = self.bytes.len();
        let padding = (align - (offset % align)) % align;

        self.bytes.extend(std::iter::repeat(0u8).take(padding));

        self.bytes.extend_from_slice(data);
    }

    fn write_data<T: bytemuck::Pod>(&mut self, data: T, align: usize) {
        self.write_slice(bytemuck::cast_slice(&[data]), align);
    }

    pub fn write_u32(&mut self, data: u32) -> &mut Self {
        self.write_data(data, 4);
        self
    }

    pub fn write_uvec2(&mut self, data: UVec2) -> &mut Self {
        self.write_data(data, 8);
        self
    }

    pub fn write_uvec3(&mut self, data: UVec3) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_uvec4(&mut self, data: UVec4) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_i32(&mut self, data: i32) -> &mut Self {
        self.write_data(data, 4);
        self
    }

    pub fn write_ivec2(&mut self, data: IVec2) -> &mut Self {
        self.write_data(data, 8);
        self
    }

    pub fn write_ivec3(&mut self, data: IVec3) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_ivec4(&mut self, data: IVec4) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_f32(&mut self, data: f32) -> &mut Self {
        self.write_data(data, 4);
        self
    }

    pub fn write_vec2(&mut self, data: Vec2) -> &mut Self {
        self.write_data(data, 8);
        self
    }

    pub fn write_vec3(&mut self, data: Vec3) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_vec4(&mut self, data: Vec4) -> &mut Self {
        self.write_data(data, 16);
        self
    }

    pub fn write_mat3(&mut self, data: Mat3) -> &mut Self {
        for i in 0..3 {
            self.write_data(data.col(i), 16);
        }

        self
    }

    pub fn write_mat4(&mut self, data: Mat4) -> &mut Self {
        for i in 0..4 {
            self.write_data(data.col(i), 16);
        }

        self
    }

    pub fn write_struct<T: AsStd140Bytes>(&mut self, data: &T) -> &mut Self {
        let data = data.as_std140();

        self.write_slice(data.as_slice(), data.alignment);
        self
    }

    pub fn align(&mut self) -> &mut Self {
        let offset = self.bytes.len();
        let padding = (self.alignment - (offset % self.alignment)) % self.alignment;

        self.bytes.extend(vec![0u8; padding]);

        self
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}
