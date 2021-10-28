use std::rc::Rc;

use wgpu::{util::DeviceExt, Color};

use crate::{Transform, Vertex};

pub(crate) type InstanceType = [[f32; 4]; 4];
pub(crate) const INSTANCE_SIZE: usize = std::mem::size_of::<InstanceType>();

pub(crate) fn instance_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
        array_stride: INSTANCE_SIZE as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    }
}

pub(crate) const QUAD_VERTICES: &[Vertex] = &[
    // Top Left
    Vertex {
        position: mint::Point2 { x: -1.0, y: 1.0 },
        uv: mint::Point2 { x: 0.0, y: 0.0 },
        color: rgb::RGB {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
    },
    // Top Right
    Vertex {
        position: mint::Point2 { x: 1.0, y: 1.0 },
        uv: mint::Point2 { x: 1.0, y: 0.0 },
        color: rgb::RGB {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
    },
    // Bottom Left
    Vertex {
        position: mint::Point2 { x: -1.0, y: -1.0 },
        uv: mint::Point2 { x: 0.0, y: 1.0 },
        color: rgb::RGB {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
    },
    // Bottom Right
    Vertex {
        position: mint::Point2 { x: 1.0, y: -1.0 },
        uv: mint::Point2 { x: 1.0, y: 1.0 },
        color: rgb::RGB {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        },
    },
];

pub(crate) const QUAD_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];

pub(crate) struct MeshBuffer {
    pub vertex: (wgpu::Buffer, u32),
    pub index: (wgpu::Buffer, u32),
}

impl MeshBuffer {
    pub fn from_slices(device: &wgpu::Device, vertex: &[Vertex], index: &[u16]) -> MeshBuffer {
        let vertex = (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Oblivion_QuadVertexBuffer"),
                contents: bytemuck::cast_slice(vertex),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            vertex.len() as u32,
        );

        let index = (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Oblivion_QuadIndexBuffer"),
                contents: bytemuck::cast_slice(index),
                usage: wgpu::BufferUsages::INDEX,
            }),
            index.len() as u32,
        );
        MeshBuffer { vertex, index }
    }
}

/// This is shared between all .draw() calls of the same renderable instance.
#[derive(Clone)]
pub(crate) struct PipelineData {
    pub mesh_buffer: Rc<MeshBuffer>,
    pub bind_group: Rc<wgpu::BindGroup>,
    pub instance_buffer: Rc<wgpu::Buffer>,
}

/// This is unique between .draw() calls
pub(crate) struct DrawData {
    pub pipeline_id: usize,
    pub transform: Transform,
}

pub(crate) struct RenderData {
    pub pipeline_data: PipelineData,
    pub instance_count: u32,
    pub instance_data: DrawData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetId {
    Screen,
    CanvasId(usize),
}

impl Default for TargetId {
    fn default() -> Self {
        Self::Screen
    }
}

#[derive(Default)]
pub(crate) struct RenderGroup {
    pub target_id: TargetId,
    pub clear_color: Option<Color>,
    pub queue: Vec<RenderData>,
}
