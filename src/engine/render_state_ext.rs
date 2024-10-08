use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use image::{EncodableLayout, ImageBuffer, Rgba};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::render_state::RenderState;

use super::{
    shader::{WgslShader, WgslShaderSource},
    texture::{WgpuTexture, WgpuTextureLoadError},
};

pub trait RenderStateExt {
    fn load_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> WgslShader;

    fn load_cubemap_texture<P: AsRef<Path> + Debug>(
        &self,
        relative_path: P,
    ) -> Result<WgpuTexture, WgpuTextureLoadError>;
}

fn read_cubemap_images(paths: &[PathBuf]) -> Vec<ImageBuffer<Rgba<f32>, Vec<f32>>> {
    (0..6)
        .into_par_iter()
        .map(|i| {
            log::info!("Reading face {} of cube map", i);
            image::open(&paths[i]).unwrap().to_rgba32f() // todo: propagate this error?
        })
        .collect()
}

impl RenderStateExt for RenderState {
    fn load_shader<P: AsRef<Path> + Debug>(&self, relative_path: P) -> WgslShader {
        let mut source = WgslShaderSource::load(relative_path);

        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut module = self.device.create_shader_module(source.desc());
        let err = pollster::block_on(self.device.pop_error_scope());

        if err.is_some() {
            source = WgslShaderSource::fallback();
            module = self.device.create_shader_module(source.desc());
        }

        WgslShader { source, module }
    }

    fn load_cubemap_texture<P: AsRef<Path> + Debug>(
        &self,
        relative_path: P,
    ) -> Result<WgpuTexture, WgpuTextureLoadError> {
        let parent_path = std::env::current_dir()?;
        let path = parent_path.join(&relative_path);

        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

        // let images = [
        //     image::open(path.join("px.hdr")).unwrap().to_rgba32f(),
        //     image::open(path.join("nx.hdr")).unwrap().to_rgba32f(),
        //     image::open(path.join("py.hdr")).unwrap().to_rgba32f(),
        //     image::open(path.join("ny.hdr")).unwrap().to_rgba32f(),
        //     image::open(path.join("pz.hdr")).unwrap().to_rgba32f(),
        //     image::open(path.join("nz.hdr")).unwrap().to_rgba32f(),
        // ];
        let faces = ["px.hdr", "nx.hdr", "py.hdr", "ny.hdr", "pz.hdr", "nz.hdr"];
        let paths = faces.map(|f| path.join(f));

        // we can't use ? operator since we're creating the array from a closure
        for path in &paths {
            if !path.exists() {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
            }
        }

        // let images: [_; 6] = std::array::from_fn(|i| {
        //     log::info!("Reading face {} of cube map", faces[i]);
        //     image::open(&paths[i]).unwrap().to_rgba32f()
        // });

        let images = read_cubemap_images(&paths);

        let format = wgpu::TextureFormat::Rgba32Float;
        let bytes_per_pixel = format.block_copy_size(None).unwrap();

        let size = wgpu::Extent3d {
            width: images[0].width(),
            height: images[0].height(),
            depth_or_array_layers: 6,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} sampler", name)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        for (index, image) in images.iter().enumerate() {
            self.queue.write_texture(
                wgpu::ImageCopyTextureBase {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_bytes(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * bytes_per_pixel),
                    rows_per_image: Some(size.height),
                },
                wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(WgpuTexture {
            name,
            texture,
            sampler,
        })
    }
}
