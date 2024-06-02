use crate::game::vertex;
use crate::render_state::{CommandEncoderResource, RenderState, SurfaceTextureResource};
use bevy_ecs::prelude::*;
use wgpu::util::DeviceExt;
use wgpu::{TextureDimension, TextureFormat};

#[derive(Resource)]
pub struct SolidTerrainRenderContext {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub texture: wgpu::Texture,
    pub pipeline: wgpu::RenderPipeline,
}

impl SolidTerrainRenderContext {
    pub fn new(render_state: &RenderState) -> Self {
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

        // Currently unused, because for debugging purposes, the world renderer just draws directly
        // to the screen right now.
        let texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Solid Terrain Texture"),
                size: wgpu::Extent3d {
                    width: render_state.size.width,
                    height: render_state.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

        let shader = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("solid_terrain.wgsl"));

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Solid Terrain Render Pipeline Layout"),
                    bind_group_layouts: &[],
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
                    depth_stencil: None,
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
                            format: render_state
                                .surface
                                .get_current_texture()
                                .unwrap()
                                .texture
                                .format(),
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview: None,
                });

        Self {
            vertex_buffer,
            index_buffer,
            texture,
            pipeline,
        }
    }

    pub fn draw(&self, surface_texture: &wgpu::SurfaceTexture, encoder: &mut wgpu::CommandEncoder) {
        let view = surface_texture.texture.create_view(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Solid Terrain Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..vertex::INDICES.len() as u32, 0, 0..1);
    }
}

pub fn init_solid_terrain_renderer(mut commands: Commands, render_state: Res<RenderState>) {
    let solid_terrain_render_context = SolidTerrainRenderContext::new(&render_state);
    commands.insert_resource(solid_terrain_render_context);
}

pub fn draw_solid_terrain(
    render_context: Res<SolidTerrainRenderContext>,
    surface_texture_resource: Res<SurfaceTextureResource>,
    mut command_encoder_resource: ResMut<CommandEncoderResource>,
) {
    render_context.draw(&surface_texture_resource, &mut command_encoder_resource);
}
