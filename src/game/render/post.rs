use crate::game::camera::CameraBuffer;
use crate::game::object::ObjectsBuffer;
use crate::game::render::world::SolidTerrainRenderContext;
use crate::game::vertex;
use crate::render_state::{
    CommandEncoderResource, RenderState, SurfaceTextureResource, WindowResizeEvent,
};
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

use super::ReloadRenderContextEvent;

#[derive(Resource)]
pub struct FullscreenQuad {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertex_shader_module: wgpu::ShaderModule,
}

impl FullscreenQuad {
    pub fn new(render_state: &RenderState) -> Self {
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Fullscreen Quad Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertex::FrameVertex::VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Fullscreen Quad Index Buffer"),
                    contents: bytemuck::cast_slice(vertex::FrameVertex::INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let vertex_shader_module = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/full_frame_vertex.wgsl"));

        Self {
            vertex_buffer,
            index_buffer,
            vertex_shader_module,
        }
    }

    pub fn init(mut commands: Commands, render_state: Res<RenderState>) {
        let fullscreen_quad = FullscreenQuad::new(&render_state);
        commands.insert_resource(fullscreen_quad);
    }
}

#[derive(Resource)]
pub struct RaytraceRenderContext {
    pub color_texture: wgpu::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub camera_uniform_bind_group: wgpu::BindGroup,
    pub objects_uniform_bind_group: wgpu::BindGroup,
}

impl RaytraceRenderContext {
    pub fn new(
        render_state: &RenderState,
        fullscreen_quad: &FullscreenQuad,
        camera_buffer: &CameraBuffer,
        objects_buffer: &ObjectsBuffer,
    ) -> Self {
        let color_texture_format = wgpu::TextureFormat::Rgba32Float;

        let color_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Raytrace Pass Color Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: color_texture_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let camera_uniform_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Raytrace Pass Camera Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let camera_uniform_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Raytrace Pass Camera Uniform Bind Group"),
                    layout: &camera_uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.buffer.as_entire_binding(),
                    }],
                });

        let objects_uniform_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Raytrace Pass Objects Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let objects_uniform_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Raytrace Pass Objects Uniform Bind Group"),
                    layout: &camera_uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: objects_buffer.buffer.as_entire_binding(),
                    }],
                });

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raytrace Pass Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_uniform_bind_group_layout,
                        &objects_uniform_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let shader_path = std::env::current_dir()
            .unwrap()
            .join("assets/raytrace.wgsl");

        let shader_src = match std::fs::read_to_string(shader_path) {
            Ok(src) => src,
            Err(_) => {
                log::warn!("Couldn't read file at assets/raytrace.wgsl, using fallback shader");
                include_str!("shaders/raytrace.wgsl").to_owned()
            }
        };

        let fragment_shader_module =
            render_state
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("raytrace.wgsl"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_src)),
                });

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Raytrace Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &fullscreen_quad.vertex_shader_module,
                        entry_point: "vertex",
                        compilation_options: Default::default(),
                        buffers: &[vertex::FrameVertex::LAYOUT],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader_module,
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: color_texture_format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        Self {
            color_texture,
            pipeline,
            camera_uniform_bind_group,
            objects_uniform_bind_group,
        }
    }

    pub fn recreate(
        &mut self,
        render_state: &RenderState,
        fullscreen_quad: &FullscreenQuad,
        camera_buffer: &CameraBuffer,
        objects_buffer: &ObjectsBuffer,
    ) {
        *self = Self::new(render_state, fullscreen_quad, camera_buffer, objects_buffer);
        log::info!("Raytrace pass render context recreated");
    }

    pub fn draw(&self, fullscreen_quad: &FullscreenQuad, encoder: &mut wgpu::CommandEncoder) {
        let color_texture_view = self
            .color_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Raytrace Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, &self.camera_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.objects_uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            fullscreen_quad.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
        objects_buffer: Res<ObjectsBuffer>,
    ) {
        let raytrace_render_context = RaytraceRenderContext::new(
            &render_state,
            &fullscreen_quad,
            &camera_buffer,
            &objects_buffer,
        );

        commands.insert_resource(raytrace_render_context);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        render_state: Res<RenderState>,
        mut raytrace_render_context: ResMut<RaytraceRenderContext>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
        objects_buffer: Res<ObjectsBuffer>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
        mut reload_events: EventReader<ReloadRenderContextEvent>,
    ) {
        for _ in resize_events.read() {
            raytrace_render_context.recreate(
                &render_state,
                &fullscreen_quad,
                &camera_buffer,
                &objects_buffer,
            );
        }

        for _ in reload_events.read() {
            raytrace_render_context.recreate(
                &render_state,
                &fullscreen_quad,
                &camera_buffer,
                &objects_buffer,
            );
        }

        raytrace_render_context.draw(&fullscreen_quad, &mut command_encoder_resource);
    }
}

#[derive(Resource)]
pub struct BloomRenderContext {
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub downsample_texture_bind_group_layout: wgpu::BindGroupLayout,
    pub downsample_texture: wgpu::Texture,

    pub downsample_texture_views: Vec<wgpu::TextureView>,
    pub downsample_texture_sampler: wgpu::Sampler,

    pub mip_levels: u32,
}

impl BloomRenderContext {
    pub fn new(
        render_state: &RenderState,
        fullscreen_quad: &FullscreenQuad,
        mip_levels: u32,
    ) -> Self {
        let downsample_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Bloom Downsample Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let downsample_texture_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Bloom Downsample Texture Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: mip_levels as f32,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                });

        let downsample_texture_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom Downsample Pass Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let shader_path = std::env::current_dir()
            .unwrap()
            .join("assets/bloom_downsample.wgsl");

        // todo: fallback shader
        let shader_src = std::fs::read_to_string(shader_path).unwrap();

        let fragment_shader_module =
            render_state
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Bloom Downsample Pass Fragment Shader Module"),
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_src)),
                });

        let downsample_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Downsample Pass Render Pipeline Layout"),
                    bind_group_layouts: &[&downsample_texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let downsample_pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Downsample Pass Render Pipeline"),
                    layout: Some(&downsample_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &fullscreen_quad.vertex_shader_module,
                        entry_point: "vertex",
                        compilation_options: Default::default(),
                        buffers: &[vertex::FrameVertex::LAYOUT],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader_module,
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });

        let mut downsample_texture_views = Vec::with_capacity(mip_levels as usize);

        for lod in 0..mip_levels {
            downsample_texture_views.push(downsample_texture.create_view(
                &wgpu::TextureViewDescriptor {
                    format: Some(wgpu::TextureFormat::Rgba32Float),
                    base_mip_level: lod,
                    mip_level_count: Some(1),
                    ..Default::default()
                },
            ));
        }

        Self {
            downsample_pipeline,
            downsample_texture_bind_group_layout,
            downsample_texture,

            downsample_texture_sampler,
            downsample_texture_views,

            mip_levels,
        }
    }

    pub fn draw_bloom_downsample(
        &self,
        render_state: &RenderState,
        fullscreen_quad: &FullscreenQuad,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for target_mip in 1..self.mip_levels as usize {
            let bind_group = render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!(
                        "Bloom Downsample Bind Group (target_mip = {})",
                        target_mip
                    )),
                    layout: &self.downsample_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.downsample_texture_views[target_mip - 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &self.downsample_texture_sampler,
                            ),
                        },
                    ],
                });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!(
                    "Bloom Downsample Render Pass (target_mip = {})",
                    target_mip
                )),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.downsample_texture_views[target_mip],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.downsample_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                fullscreen_quad.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);
        }
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        fullscreen_quad: Res<FullscreenQuad>,
    ) {
        let bloom_render_context = BloomRenderContext::new(&render_state, &fullscreen_quad, 7);
        commands.insert_resource(bloom_render_context);
    }

    pub fn update(
        render_state: Res<RenderState>,
        mut bloom_render_context: ResMut<BloomRenderContext>,
        fullscreen_quad: Res<FullscreenQuad>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
        mut reload_events: EventReader<ReloadRenderContextEvent>,
    ) {
        for _ in resize_events.read() {
            *bloom_render_context = BloomRenderContext::new(&render_state, &fullscreen_quad, 7);
            log::info!("Bloom render context recreated");
        }

        for _ in reload_events.read() {
            *bloom_render_context = BloomRenderContext::new(&render_state, &fullscreen_quad, 7);
            log::info!("Bloom render context recreated");
        }

        bloom_render_context.draw_bloom_downsample(
            &render_state,
            &fullscreen_quad,
            &mut command_encoder_resource,
        );
    }
}

#[derive(Resource)]
pub struct FinalRenderContext {
    pub pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub uniform_bind_group: wgpu::BindGroup,
}

impl FinalRenderContext {
    pub fn new(
        render_state: &RenderState,
        surface_texture: &wgpu::SurfaceTexture,
        fullscreen_quad: &FullscreenQuad,
        input_color_texture: &wgpu::Texture,
        camera_buffer: &CameraBuffer,
    ) -> Self {
        let input_color_texture_view =
            input_color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let input_color_texture_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Final Pass Input Color Texture Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let texture_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final Pass Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });

        let texture_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Final Pass Input Texture Bind Group"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&input_color_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&input_color_texture_sampler),
                        },
                    ],
                });

        let uniform_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final Pass Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let uniform_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Final Pass Uniform Bind Group"),
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.buffer.as_entire_binding(),
                    }],
                });

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Final Pass Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let fragment_shader_module = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/final.wgsl"));

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Final Pass Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &fullscreen_quad.vertex_shader_module,
                        entry_point: "vertex",
                        compilation_options: Default::default(),
                        buffers: &[vertex::FrameVertex::LAYOUT],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fragment_shader_module,
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_texture.texture.format(),
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        Self {
            pipeline,
            texture_bind_group,
            uniform_bind_group,
        }
    }

    pub fn recreate(
        &mut self,
        render_state: &RenderState,
        surface_texture: &wgpu::SurfaceTexture,
        fullscreen_quad: &FullscreenQuad,
        input_color_texture: &wgpu::Texture,
        camera_buffer: &CameraBuffer,
    ) {
        *self = Self::new(
            render_state,
            surface_texture,
            fullscreen_quad,
            input_color_texture,
            camera_buffer,
        );
        log::info!("Final pass render context recreated");
    }

    pub fn draw(
        &self,
        surface_texture: &wgpu::SurfaceTexture,
        fullscreen_quad: &FullscreenQuad,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Final Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            fullscreen_quad.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        raytrace_render_context: Res<RaytraceRenderContext>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
    ) {
        let final_render_context = FinalRenderContext::new(
            &render_state,
            &render_state.surface.get_current_texture().unwrap(),
            &fullscreen_quad,
            &raytrace_render_context.color_texture,
            &camera_buffer,
        );

        commands.insert_resource(final_render_context);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        render_state: Res<RenderState>,
        mut final_render_context: ResMut<FinalRenderContext>,
        surface_texture_resource: Res<SurfaceTextureResource>,
        fullscreen_quad: Res<FullscreenQuad>,
        raytrace_render_context: Res<RaytraceRenderContext>,
        camera_buffer: Res<CameraBuffer>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
        mut reload_events: EventReader<ReloadRenderContextEvent>,
    ) {
        for _ in resize_events.read() {
            final_render_context.recreate(
                &render_state,
                &surface_texture_resource,
                &fullscreen_quad,
                &raytrace_render_context.color_texture,
                &camera_buffer,
            );
        }

        for _ in reload_events.read() {
            final_render_context.recreate(
                &render_state,
                &surface_texture_resource,
                &fullscreen_quad,
                &raytrace_render_context.color_texture,
                &camera_buffer,
            );
        }

        final_render_context.draw(
            &surface_texture_resource,
            &fullscreen_quad,
            &mut command_encoder_resource,
        );
    }
}
