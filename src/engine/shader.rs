use std::{
    borrow::Cow,
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::util;

pub enum WgslShaderSource {
    File {
        name: String,
        source: String,
        path: PathBuf,
    },
    Fallback,
}

impl WgslShaderSource {
    fn read_source<P: AsRef<Path>>(relative_path: P) -> Result<Self, std::io::Error> {
        let parent_path = std::env::current_dir()?;
        let path = parent_path.join(relative_path);

        let source = std::fs::read_to_string(&path)?;
        let source = util::preprocess::resolve_includes(source, &parent_path)?;

        let name = path.file_name().unwrap().to_str().unwrap().to_owned(); // ew

        Ok(Self::File { name, source, path })
    }

    pub fn load<P: AsRef<Path> + Debug>(relative_path: P) -> Self {
        match Self::read_source(&relative_path) {
            Ok(s) => s,
            Err(_) => {
                log::warn!(
                    "Shader at path {:?} failed to load, substituting fallback shader.",
                    relative_path
                );
                Self::Fallback
            }
        }
    }

    pub fn fallback() -> Self {
        Self::Fallback
    }

    pub fn name(&self) -> &str {
        match self {
            WgslShaderSource::File {
                name,
                source: _,
                path: _,
            } => name,
            WgslShaderSource::Fallback => "fallback.wgsl",
        }
    }

    pub fn desc(&self) -> wgpu::ShaderModuleDescriptor {
        match self {
            WgslShaderSource::File {
                name,
                source,
                path: _,
            } => wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            },
            WgslShaderSource::Fallback => wgpu::ShaderModuleDescriptor {
                label: Some("Fallback Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "assets/fallback.wgsl"
                ))),
            },
        }
    }

    pub fn is_fallback(&self) -> bool {
        matches!(self, WgslShaderSource::Fallback)
    }
}

pub struct WgslShader {
    pub(in crate::engine) source: WgslShaderSource,
    pub(in crate::engine) module: wgpu::ShaderModule,
}

impl WgslShader {
    pub fn source(&self) -> &WgslShaderSource {
        &self.source
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}
