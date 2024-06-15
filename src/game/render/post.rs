use crate::game::camera::CameraBuffer;
use crate::game::render::world::SolidTerrainRenderContext;
use crate::game::vertex;
use crate::render_state::{
    CommandEncoderResource, RenderState, SurfaceTextureResource, WindowResizeEvent,
};
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;

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
        input_depth_texture: &wgpu::Texture,
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
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                });

        let input_depth_texture_view =
            input_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let input_depth_texture_sampler =
            render_state
                .device
                .create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Final Pass Input Depth Texture Sampler"),
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    compare: None,
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
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
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&input_depth_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&input_depth_texture_sampler),
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

    pub fn resize(
        &mut self,
        render_state: &RenderState,
        surface_texture: &wgpu::SurfaceTexture,
        fullscreen_quad: &FullscreenQuad,
        input_color_texture: &wgpu::Texture,
        input_depth_texture: &wgpu::Texture,
        camera_buffer: &CameraBuffer,
    ) {
        *self = Self::new(
            render_state,
            surface_texture,
            fullscreen_quad,
            input_color_texture,
            input_depth_texture,
            camera_buffer,
        );
        log::info!("Final pass render context resized");
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
        solid_terrain_render_context: Res<SolidTerrainRenderContext>,
        fullscreen_quad: Res<FullscreenQuad>,
        camera_buffer: Res<CameraBuffer>,
    ) {
        let final_render_context = FinalRenderContext::new(
            &render_state,
            &render_state.surface.get_current_texture().unwrap(),
            &fullscreen_quad,
            &solid_terrain_render_context.color_texture,
            &solid_terrain_render_context.depth_texture,
            &camera_buffer,
        );

        commands.insert_resource(final_render_context);
    }

    pub fn update(
        render_state: Res<RenderState>,
        mut final_render_context: ResMut<FinalRenderContext>,
        surface_texture_resource: Res<SurfaceTextureResource>,
        fullscreen_quad: Res<FullscreenQuad>,
        solid_terrain_render_context: Res<SolidTerrainRenderContext>,
        camera_buffer: Res<CameraBuffer>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
    ) {
        for _ in resize_events.read() {
            final_render_context.resize(
                &render_state,
                &surface_texture_resource,
                &fullscreen_quad,
                &solid_terrain_render_context.color_texture,
                &solid_terrain_render_context.depth_texture,
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
