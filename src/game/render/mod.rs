use bevy_ecs::prelude::*;

pub mod post;
pub mod world;

#[derive(Event)]
pub struct ReloadRenderContextEvent;

// pub struct TextureConfig {
//     name: String,
//     size: wgpu::Extent3d,
//     mip_level_count: u32,
//     format: wgpu::TextureFormat,
// }

// pub struct PassConfig {}

// pub struct Renderer {
//     textures: HashMap<String, wgpu::Texture>,
// }

// impl Renderer {
//     pub fn create_texture(&mut self, name: &str, config: TextureConfig) {
//         // self.textures.insert(config.name, )
//     }
// }

// pub struct FullscreenPassBuilder<'a> {
//     render_state: &'a RenderState,
//     fullscreen_quad: &'a FullscreenQuad,
//     buffers: Vec<wgpu::Buffer>,
// }

// impl<'a> FullscreenPassBuilder<'a> {
//     pub fn new(render_state: &'a RenderState, fullscreen_quad: &'a FullscreenQuad) -> Self {
//         todo!()
//     }

//     pub fn init_uniform_bind_group(&mut self, binding: u32) {}

//     pub fn set_uniform<T>(&mut self, location: u32, value: T)
//     where
//         T: bytemuck::Pod + bytemuck::Zeroable,
//     {
//         let buffer =
//             self.render_state
//                 .device
//                 .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                     label: None,
//                     contents: bytemuck::cast_slice(&[value]),
//                     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//                 });
//     }

//     pub fn init_texture_bind_group(&mut self, binding: u32) {}

//     pub fn set_shader(&mut self, shader_path: &Path) {}
// }
