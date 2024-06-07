use crate::game::camera::CameraBuffer;
use crate::game::vertex;
use crate::render_state::{CommandEncoderResource, RenderState, WindowResizeEvent};
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;
use wgpu::TextureDimension;

#[derive(Resource)]
pub struct SolidTerrainRenderContext {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub color_texture: wgpu::Texture,
    pub depth_texture: wgpu::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group: wgpu::BindGroup,
}

impl SolidTerrainRenderContext {
    pub fn new(render_state: &RenderState, camera_buffer: &CameraBuffer) -> Self {
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Solid Terrain Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertex::VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Solid Terrain Index Buffer"),
                    contents: bytemuck::cast_slice(vertex::INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let color_texture_format = wgpu::TextureFormat::Rgba8Unorm;
        let depth_texture_format = wgpu::TextureFormat::Depth32Float;

        // Currently unused, because for debugging purposes, the world renderer just draws directly
        // to the screen right now.
        let color_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Solid Terrain Color Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: color_texture_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let depth_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Solid Terrain Depth Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: depth_texture_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

        let shader = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/solid_terrain.wgsl"));

        let uniform_bind_group_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Solid Terrain Uniform Bind Group Layout"),
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
                    label: Some("Solid Terrain Uniform Bind Group"),
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
                    label: Some("Solid Terrain Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Solid Terrain Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vertex",
                        compilation_options: Default::default(),
                        buffers: &[vertex::BlockVertex::LAYOUT],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: depth_texture_format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fragment",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: color_texture_format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        Self {
            vertex_buffer,
            index_buffer,
            color_texture,
            depth_texture,
            pipeline,
            uniform_bind_group,
        }
    }

    pub fn resize(&mut self, render_state: &RenderState, camera_buffer: &CameraBuffer) {
        *self = Self::new(render_state, camera_buffer);
        log::info!("Solid terrain render context resized");
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        let color_texture_view = self
            .color_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Solid Terrain Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..vertex::INDICES.len() as u32, 0, 0..1);
    }

    pub fn init(
        mut commands: Commands,
        render_state: Res<RenderState>,
        camera_buffer: Res<CameraBuffer>,
    ) {
        let solid_terrain_render_context =
            SolidTerrainRenderContext::new(&render_state, &camera_buffer);
        commands.insert_resource(solid_terrain_render_context);
    }

    pub fn update(
        render_state: Res<RenderState>,
        camera_buffer: Res<CameraBuffer>,
        mut render_context: ResMut<SolidTerrainRenderContext>,
        mut command_encoder_resource: ResMut<CommandEncoderResource>,
        mut resize_events: EventReader<WindowResizeEvent>,
    ) {
        for _ in resize_events.read() {
            // Reconfigure the render context when the screen is resized
            render_context.resize(&render_state, &camera_buffer);
        }

        render_context.draw(&mut command_encoder_resource);
    }
}
