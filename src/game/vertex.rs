use glam::{Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlockVertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    color: Vec3,
}

impl BlockVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<BlockVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
            3 => Float32x3,
        ],
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameVertex {
    position: Vec2,
    uv: Vec2,
}

impl FrameVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FrameVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    };

    pub const VERTICES: &'static [FrameVertex] = &[
        FrameVertex {
            position: Vec2::new(-1.0, -1.0),
            uv: Vec2::new(0.0, 0.0),
        },
        FrameVertex {
            position: Vec2::new(1.0, -1.0),
            uv: Vec2::new(1.0, 0.0),
        },
        FrameVertex {
            position: Vec2::new(1.0, 1.0),
            uv: Vec2::new(1.0, 1.0),
        },
        FrameVertex {
            position: Vec2::new(-1.0, 1.0),
            uv: Vec2::new(0.0, 1.0),
        },
    ];

    pub const INDICES: &'static [u16] = &[0, 1, 2, 0, 2, 3];
}

pub const VERTICES: &[BlockVertex] = &[
    BlockVertex {
        position: Vec3::new(0.0, 0.5, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        normal: Vec3::ZERO,
        uv: Vec2::ZERO,
    },
    BlockVertex {
        position: Vec3::new(-0.5, -0.5, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        normal: Vec3::ZERO,
        uv: Vec2::ZERO,
    },
    BlockVertex {
        position: Vec3::new(0.5, -0.5, 0.0),
        color: Vec3::new(0.0, 0.0, 1.0),
        normal: Vec3::ZERO,
        uv: Vec2::ZERO,
    },
];

pub const INDICES: &[u16] = &[0, 1, 2];
