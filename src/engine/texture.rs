use std::{error::Error, fmt::Display};

use derived_deref::Deref;

pub type HdrImageBuffer = image::ImageBuffer<image::Rgba<f32>, Vec<f32>>;

#[derive(Debug)]
pub enum WgpuTextureLoadError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
}

impl From<std::io::Error> for WgpuTextureLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<image::ImageError> for WgpuTextureLoadError {
    fn from(value: image::ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl Display for WgpuTextureLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Error for WgpuTextureLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WgpuTextureLoadError::IoError(error) => Some(error),
            WgpuTextureLoadError::ImageError(error) => Some(error),
        }
    }
}

#[derive(Deref)]
pub struct WgpuTexture {
    pub(in crate::engine) name: String,

    #[target]
    pub(in crate::engine) texture: wgpu::Texture,
    pub(in crate::engine) sampler: wgpu::Sampler,
}

impl WgpuTexture {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
