use crate::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex for MeshVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/*pub const QUAD_VERTICES: &[MeshVertex] = &[
    // Top Left
    MeshVertex {
        position: [-0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    // Top Right
    MeshVertex {
        position: [0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    // Bottom Left
    MeshVertex {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    // Bottom Right
    MeshVertex {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];*/
