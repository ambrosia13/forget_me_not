use bevy_ecs::prelude::Event;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use derived_deref::{Deref, DerefMut};
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::window::Window;

#[derive(Resource)]
pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused)]
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
}

pub fn begin_frame(world: &mut World) -> Result<(), wgpu::SurfaceError> {
    let render_state = world.resource::<RenderState>();

    let encoder = render_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    let surface_texture = render_state.surface.get_current_texture()?;

    world.insert_resource(CommandEncoderResource(encoder));
    world.insert_resource(SurfaceTextureResource(surface_texture));

    Ok(())
}

pub fn finish_frame(world: &mut World) {
    let CommandEncoderResource(encoder) =
        world.remove_resource::<CommandEncoderResource>().unwrap();
    let SurfaceTextureResource(surface_texture) =
        world.remove_resource::<SurfaceTextureResource>().unwrap();

    let render_state = world.resource::<RenderState>();

    render_state.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();
}

#[derive(Resource, Deref, DerefMut)]
pub struct CommandEncoderResource(pub wgpu::CommandEncoder);

#[derive(Resource, Deref, DerefMut)]
pub struct SurfaceTextureResource(pub wgpu::SurfaceTexture);

#[derive(Event, Deref, DerefMut)]
pub struct ResizeEvent(pub winit::dpi::PhysicalSize<u32>);
