#![warn(clippy::clone_on_ref_ptr)]
use std::rc::Rc;

pub use crate::{context::*, renderables::*, shader::*};
use wgpu::{util::DeviceExt, Color};

mod context;
pub(crate) mod helpers;
mod renderables;
mod shader;

pub(crate) const QUAD_VERTICES: &[Vertex] = &[
    // Top Left
    Vertex {
        position: [-1.0, 1.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    // Top Right
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    // Bottom Left
    Vertex {
        position: [-1.0, -1.0],
        uv: [0.0, 1.0],
        color: [1.0, 1.0, 1.0],
    },
    // Bottom Right
    Vertex {
        position: [1.0, -1.0],
        uv: [1.0, 1.0],
        color: [1.0, 1.0, 1.0],
    },
];

pub(crate) const QUAD_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 3]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }
}

/// This is shared between all .draw() calls of the same renderable instance.
pub(crate) struct PipelineData {
    pub mesh_buffer: Rc<MeshBuffer>,
    pub bind_group: wgpu::BindGroup,
}

/// This is unique between .draw() calls
pub(crate) struct InstanceData {
    pub pipeline_id: usize,
    pub transform: Transform,
}

pub(crate) struct RenderData {
    pipeline_data: Rc<PipelineData>,
    instance_data: InstanceData,
}

#[derive(Default)]
pub struct Render {
    clear_color: Option<Color>,
    shader_queue: Vec<usize>,
    queue: Vec<RenderData>,
}

impl Render {
    pub fn new() -> Self {
        Render::default()
    }

    pub(crate) fn push_data(&mut self, pipeline_data: Rc<PipelineData>, transform: Transform) {
        self.queue.push(RenderData {
            pipeline_data,
            instance_data: InstanceData {
                pipeline_id: self.shader_queue.last().copied().unwrap_or(0),
                transform,
            },
        })
    }
}

pub fn clear(render: &mut Render, color: wgpu::Color) {
    render.clear_color = Some(color);
    // We also need to clear the draw queue to give the illusion of the clear color overwriting everything else
    // This should produce the same behavior as just overwriting everything else though.
    render.queue.clear();
}

pub fn push_shader(render: &mut Render, shader: &Shader) {
    render.shader_queue.push(shader.pipeline_id);
}

pub fn pop_shader(render: &mut Render) {
    render.shader_queue.pop();
}
