use crate::engine::render_state_ext::RenderStateExt;
use crate::engine::WgpuResourceRegistry;
use crate::game::camera::CameraBuffer;
use crate::game::object::ObjectsBuffer;
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
    pub color_texture_copy: wgpu::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub camera_uniform_bind_group: wgpu::BindGroup,
    pub objects_uniform_bind_group: wgpu::BindGroup,
    pub texture_bind_group: wgpu::BindGroup,
}

impl RaytraceRenderContext {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

    pub fn new(
        render_state: &RenderState,
        resource_registry: &mut WgpuResourceRegistry,
        fullscreen_quad: &FullscreenQuad,
        camera_buffer: &CameraBuffer,
        objects_buffer: &ObjectsBuffer,
    ) -> Self {
        let color_texture_desc = wgpu::TextureDescriptor {
            label: Some("Raytrace Pass Color Texture"),
            size: wgpu::Extent3d {
                width: render_state.size.width,
                height: render_state.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: RaytraceRenderContext::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let color_texture = render_state.device.create_texture(&color_texture_desc);
        let color_texture_copy = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Raytrace Pass Color Texture Copy"),
                ..color_texture_desc
            });

        let color_texture_copy_view =
            color_texture_copy.create_view(&wgpu::TextureViewDescriptor::default());
        let color_texture_copy_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Color Texture Copy Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
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
                    layout: &objects_uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: objects_buffer.buffer.as_entire_binding(),
                    }],
                });

        let cubemap_texture = resource_registry
            .get_or_create_texture(render_state, "assets/textures/cubemaps/sunset")
            .unwrap();

        let cubemap_view = cubemap_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let texture_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Raytrace Pass Texture Bind Group Layout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::Cube,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let texture_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Raytrace Pass Texture Bind Group"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&color_texture_copy_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&color_texture_copy_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&cubemap_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(cubemap_texture.sampler()),
                        },
                    ],
                });

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Raytrace Pass Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_uniform_bind_group_layout,
                        &objects_uniform_bind_group_layout,
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let fragment_shader = render_state.load_shader("assets/shaders/raytrace.wgsl");

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
                        module: fragment_shader.module(),
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: RaytraceRenderContext::TEXTURE_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        Self {
            color_texture,
            color_texture_copy,
            pipeline,
            camera_uniform_bind_group,
            objects_uniform_bind_group,
            texture_bind_group,
        }
    }

    pub fn recreate(
        &mut self,
        render_state: &RenderState,
        resource_registry: &mut WgpuResourceRegistry,
        fullscreen_quad: &FullscreenQuad,
        camera_buffer: &CameraBuffer,
        objects_buffer: &ObjectsBuffer,
    ) {
        *self = Self::new(
            render_state,
            resource_registry,
            fullscreen_quad,
            camera_buffer,
            objects_buffer,
        );
    }

    pub fn draw(&self, fullscreen_quad: &FullscreenQuad, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTextureBase {
                texture: &self.color_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTextureBase {
                texture: &self.color_texture_copy,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.color_texture.size(),
        );

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
        render_pass.set_bind_group(2, &self.texture_bind_group, &[]);

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
        mut resource_registry: ResMut<WgpuResourceRegistry>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
        objects_buffer: Res<ObjectsBuffer>,
    ) {
        let raytrace_render_context = RaytraceRenderContext::new(
            &render_state,
            resource_registry.as_mut(),
            &fullscreen_quad,
            &camera_buffer,
            &objects_buffer,
        );

        commands.insert_resource(raytrace_render_context);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        render_state: Res<RenderState>,
        mut resource_registry: ResMut<WgpuResourceRegistry>,
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
                &mut resource_registry,
                &fullscreen_quad,
                &camera_buffer,
                &objects_buffer,
            );
        }

        for _ in reload_events.read() {
            raytrace_render_context.recreate(
                &render_state,
                &mut resource_registry,
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
    // downsampling
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub downsample_texture_bind_groups: Vec<wgpu::BindGroup>,
    pub downsample_texture: wgpu::Texture,
    pub downsample_texture_views: Vec<wgpu::TextureView>,

    // upsampling
    pub first_upsample_pipeline: wgpu::RenderPipeline,
    pub first_upsample_texture_bind_group: wgpu::BindGroup,
    pub upsample_pipeline: wgpu::RenderPipeline,
    pub upsample_texture_bind_groups: Vec<wgpu::BindGroup>,
    pub upsample_texture: wgpu::Texture,
    pub upsample_texture_views: Vec<wgpu::TextureView>,

    // merge to final image
    pub bloom_texture: wgpu::Texture,
    pub merge_pipeline: wgpu::RenderPipeline,
    pub merge_texture_bind_group: wgpu::BindGroup,

    pub mip_levels: u32,
}

impl BloomRenderContext {
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg11b10Float;
    pub const ADDRESS_MODE: wgpu::AddressMode = wgpu::AddressMode::ClampToBorder;

    pub fn new(
        render_state: &RenderState,
        fullscreen_quad: &FullscreenQuad,
        input_texture: &wgpu::Texture,
        camera_buffer: &CameraBuffer,
    ) -> Self {
        let input_texture_size = u32::min(input_texture.width(), input_texture.height());
        let max_possible_mip_levels = f32::log2(input_texture_size as f32) as u32;

        // use as many mip levels as we can
        let mip_levels = max_possible_mip_levels;

        let push_constant_range = wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::FRAGMENT,
            range: 0..8, // first 4 bytes current_lod, next 4 bytes max_lod
        };

        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(input_texture.format()),
            ..Default::default()
        });

        let input_texture_sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Bloom Downsample Input Texture Sampler"),
                address_mode_u: BloomRenderContext::ADDRESS_MODE,
                address_mode_v: BloomRenderContext::ADDRESS_MODE,
                address_mode_w: BloomRenderContext::ADDRESS_MODE,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

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
                format: BloomRenderContext::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let downsample_texture_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Bloom Downsample Texture Sampler"),
                    address_mode_u: BloomRenderContext::ADDRESS_MODE,
                    address_mode_v: BloomRenderContext::ADDRESS_MODE,
                    address_mode_w: BloomRenderContext::ADDRESS_MODE,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: mip_levels as f32,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                });

        let mut downsample_texture_views = Vec::with_capacity(mip_levels as usize);

        for lod in 0..mip_levels {
            downsample_texture_views.push(downsample_texture.create_view(
                &wgpu::TextureViewDescriptor {
                    format: Some(downsample_texture.format()),
                    base_mip_level: lod,
                    mip_level_count: Some(1),
                    ..Default::default()
                },
            ));
        }

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
                        // Camera uniform
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let mut downsample_texture_bind_groups = Vec::with_capacity(mip_levels as usize);

        downsample_texture_bind_groups.push(render_state.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bloom Downsample Bind Group (target_mip = 0)"),
                layout: &downsample_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&input_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&input_texture_sampler),
                    },
                    // Camera uniform
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: camera_buffer.buffer.as_entire_binding(),
                    },
                ],
            },
        ));

        for target_mip in 1..mip_levels as usize {
            downsample_texture_bind_groups.push(render_state.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(&format!(
                        "Bloom Downsample Bind Group (target_mip = {})",
                        target_mip
                    )),
                    layout: &downsample_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &downsample_texture_views[target_mip - 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&downsample_texture_sampler),
                        },
                        // Camera uniform
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: camera_buffer.buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        let downsample_shader = render_state.load_shader("assets/shaders/bloom_downsample.wgsl");

        let downsample_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Downsample Pass Render Pipeline Layout"),
                    bind_group_layouts: &[&downsample_texture_bind_group_layout],
                    push_constant_ranges: &[push_constant_range.clone()],
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
                        module: downsample_shader.module(),
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: BloomRenderContext::TEXTURE_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });

        let upsample_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Bloom Upsample Color Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BloomRenderContext::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let upsample_texture_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Bloom Upsample Texture Sampler"),
                    address_mode_u: BloomRenderContext::ADDRESS_MODE,
                    address_mode_v: BloomRenderContext::ADDRESS_MODE,
                    address_mode_w: BloomRenderContext::ADDRESS_MODE,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: mip_levels as f32,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                });

        let mut upsample_texture_views = Vec::with_capacity(mip_levels as usize);

        for target_mip in 0..mip_levels {
            upsample_texture_views.push(upsample_texture.create_view(
                &wgpu::TextureViewDescriptor {
                    format: Some(upsample_texture.format()),
                    base_mip_level: target_mip,
                    mip_level_count: Some(1),
                    ..Default::default()
                },
            ));
        }

        let first_upsample_texture_bind_group_layout = render_state
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("First Bloom Upsample Pass Bind Group Layout"),
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

        let first_upsample_texture_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("First Bloom Upsample Pass Texture Bind Group"),
                    layout: &first_upsample_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                // We're reading in the last mip of the downsample pass to begin the upsample process
                                &downsample_texture_views[mip_levels as usize - 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&downsample_texture_sampler),
                        },
                    ],
                });

        let first_upsample_shader =
            render_state.load_shader("assets/shaders/bloom_upsample_first.wgsl");

        let first_upsample_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("First Bloom Upsample Pass Pipeline Layout"),
                    bind_group_layouts: &[&first_upsample_texture_bind_group_layout],
                    push_constant_ranges: &[push_constant_range.clone()],
                });

        let first_upsample_pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("First Bloom Upsample Pass Render Pipeline"),
                    layout: Some(&first_upsample_pipeline_layout),
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
                        module: first_upsample_shader.module(),
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: BloomRenderContext::TEXTURE_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });

        let upsample_texture_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom Upsample Pass Texture Bind Group Layout"),
                    entries: &[
                        // prior mip (current + 1)
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
                        // current mip
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let upsample_shader = render_state.load_shader("assets/shaders/bloom_upsample.wgsl");

        let upsample_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Upsample Pass Pipeline Layout"),
                    bind_group_layouts: &[&upsample_texture_bind_group_layout],
                    push_constant_ranges: &[push_constant_range.clone()],
                });

        let upsample_pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Upsample Pass Render Pipeline"),
                    layout: Some(&upsample_pipeline_layout),
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
                        module: upsample_shader.module(),
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: BloomRenderContext::TEXTURE_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });

        let mut upsample_texture_bind_groups = Vec::with_capacity(mip_levels as usize);

        for target_mip in 0..(mip_levels as usize - 1) {
            upsample_texture_bind_groups.push(render_state.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(&format!(
                        "Bloom Upsample Bind Group (target_mip = {})",
                        target_mip
                    )),
                    layout: &upsample_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                // Sample current mip + 1...
                                &upsample_texture_views[target_mip + 1],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&upsample_texture_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                // ...and merge with corresponding mip of downsample texture
                                &downsample_texture_views[target_mip],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&downsample_texture_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: camera_buffer.buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        let bloom_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Bloom Color Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BloomRenderContext::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let merge_texture_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bloom Merge Pass Texture Bind Group Layout"),
                    entries: &[
                        // Scene color
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
                        // Bloom upsample color
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let merge_texture_bind_group =
            render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bloom Merge Pass Texture Bind Group"),
                    layout: &merge_texture_bind_group_layout,
                    entries: &[
                        // input texture
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&input_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&input_texture_sampler),
                        },
                        // upsample texture
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                // we want to read from the fully upsampled bloom image
                                &upsample_texture_views[0],
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&upsample_texture_sampler),
                        },
                    ],
                });

        let merge_shader = render_state.load_shader("assets/shaders/bloom_merge.wgsl");

        let merge_pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Bloom Merge Pass Pipeline Layout"),
                    bind_group_layouts: &[&merge_texture_bind_group_layout],
                    push_constant_ranges: &[push_constant_range.clone()],
                });

        let merge_pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Bloom Merge Pass Render Pipeline"),
                    layout: Some(&merge_pipeline_layout),
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
                        module: merge_shader.module(),
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: BloomRenderContext::TEXTURE_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });

        Self {
            downsample_pipeline,
            downsample_texture_bind_groups,
            downsample_texture,
            downsample_texture_views,

            first_upsample_pipeline,
            first_upsample_texture_bind_group,
            upsample_pipeline,
            upsample_texture_bind_groups,
            upsample_texture,
            upsample_texture_views,

            bloom_texture,
            merge_pipeline,
            merge_texture_bind_group,

            mip_levels,
        }
    }

    pub fn draw_bloom_downsample(
        &self,
        fullscreen_quad: &FullscreenQuad,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        for target_mip in 0..self.mip_levels as usize {
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
            render_pass.set_bind_group(0, &self.downsample_texture_bind_groups[target_mip], &[]);

            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::cast_slice(&[target_mip as u32, self.mip_levels]),
            );

            render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                fullscreen_quad.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);
        }
    }

    pub fn draw_bloom_upsample(
        &self,
        fullscreen_quad: &FullscreenQuad,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("First Bloom Upsample Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.upsample_texture_views[self.mip_levels as usize - 1],
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

        render_pass.set_pipeline(&self.first_upsample_pipeline);
        render_pass.set_bind_group(0, &self.first_upsample_texture_bind_group, &[]);

        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::cast_slice(&[self.mip_levels - 1, self.mip_levels]),
        );

        render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            fullscreen_quad.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);

        drop(render_pass);

        for target_mip in (0..(self.mip_levels as usize - 1)).rev() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!(
                    "Bloom Upsample Render Pass (target_mip = {})",
                    target_mip
                )),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.upsample_texture_views[target_mip],
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

            render_pass.set_pipeline(&self.upsample_pipeline);
            render_pass.set_bind_group(0, &self.upsample_texture_bind_groups[target_mip], &[]);

            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::cast_slice(&[target_mip as u32, self.mip_levels]),
            );

            render_pass.set_vertex_buffer(0, fullscreen_quad.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                fullscreen_quad.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..vertex::FrameVertex::INDICES.len() as u32, 0, 0..1);
        }
    }

    pub fn draw_bloom_merge(
        &self,
        fullscreen_quad: &FullscreenQuad,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let bloom_texture_view = self
            .bloom_texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.bloom_texture.format()),
                base_mip_level: 0,
                mip_level_count: None,
                ..Default::default()
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Merge Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &bloom_texture_view,
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

        render_pass.set_pipeline(&self.merge_pipeline);
        render_pass.set_bind_group(0, &self.merge_texture_bind_group, &[]);

        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::cast_slice(&[0u32, self.mip_levels]),
        );

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
        raytrace_render_context: Res<RaytraceRenderContext>,
        camera_buffer: Res<CameraBuffer>,
    ) {
        let bloom_render_context = BloomRenderContext::new(
            &render_state,
            &fullscreen_quad,
            &raytrace_render_context.color_texture,
            &camera_buffer,
        );
        commands.insert_resource(bloom_render_context);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        render_state: Res<RenderState>,
        fullscreen_quad: Res<FullscreenQuad>,

        camera_buffer: Res<CameraBuffer>,
        mut bloom_render_context: ResMut<BloomRenderContext>,
        raytrace_render_context: Res<RaytraceRenderContext>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
        mut reload_events: EventReader<ReloadRenderContextEvent>,
    ) {
        for _ in resize_events.read() {
            *bloom_render_context = BloomRenderContext::new(
                &render_state,
                &fullscreen_quad,
                &raytrace_render_context.color_texture,
                &camera_buffer,
            );
        }

        for _ in reload_events.read() {
            *bloom_render_context = BloomRenderContext::new(
                &render_state,
                &fullscreen_quad,
                &raytrace_render_context.color_texture,
                &camera_buffer,
            );
        }

        bloom_render_context.draw_bloom_downsample(&fullscreen_quad, &mut command_encoder_resource);

        bloom_render_context.draw_bloom_upsample(&fullscreen_quad, &mut command_encoder_resource);

        bloom_render_context.draw_bloom_merge(&fullscreen_quad, &mut command_encoder_resource);
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
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
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

        let fragment_shader = render_state.load_shader("assets/shaders/final.wgsl");

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
                        module: fragment_shader.module(),
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
        bloom_render_context: Res<BloomRenderContext>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
    ) {
        let final_render_context = FinalRenderContext::new(
            &render_state,
            &render_state.surface.get_current_texture().unwrap(),
            &fullscreen_quad,
            &bloom_render_context.bloom_texture,
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
        bloom_render_context: Res<BloomRenderContext>,
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
                &bloom_render_context.bloom_texture,
                &camera_buffer,
            );
        }

        for _ in reload_events.read() {
            final_render_context.recreate(
                &render_state,
                &surface_texture_resource,
                &fullscreen_quad,
                &bloom_render_context.bloom_texture,
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
