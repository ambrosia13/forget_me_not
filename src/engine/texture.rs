use std::{
    error::Error,
    fmt::{Debug, Display},
    path::{Path, PathBuf},
};

use derived_deref::Deref;

// pub enum TextureSource {
//     File { name: String, path: PathBuf },
//     Fallback,
// }

// impl TextureSource {
//     fn read_file<P: AsRef<Path>>(relative_path: P) -> Result<Self, std::io::Error> {
//         let parent_path = std::env::current_dir()?;
//         let path = parent_path.join(relative_path);

//         path.try_exists()?;

//         let name = path.file_name().unwrap().to_str().unwrap().to_owned();

//         Ok(Self::File { name, path })
//     }

//     pub fn load<P: AsRef<Path> + Debug>(relative_path: P) -> Self {
//         match Self::read_file(&relative_path) {
//             Ok(s) => s,
//             Err(_) => {
//                 log::warn!(
//                     "Texture at path {:?} failed to load, substituting fallback texture.",
//                     relative_path
//                 );
//                 Self::Fallback
//             }
//         }
//     }

//     pub fn fallback() -> Self {
//         Self::Fallback
//     }
// }

// pub struct TextureData {
//     source: TextureSource,
//     extent: wgpu::Extent3d,
// }

// impl TextureData {
//     pub fn load<P: AsRef<Path> + Debug>(relative_path: P) -> Self {
//         let source =
//     }
// }

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
