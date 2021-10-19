use crate::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex for ImageVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const IMAGE_VERTICES: &[ImageVertex] = &[
    // Top Left
    ImageVertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
    // Top Right
    ImageVertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
    // Bottom Left
    ImageVertex {
        position: [-0.5, -0.5],
        uv: [0.0, 1.0],
    },
    // Bottom Right
    ImageVertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
];

pub const IMAGE_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];
