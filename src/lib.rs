#![warn(clippy::clone_on_ref_ptr)]
use std::rc::Rc;

pub use crate::{context::*, renderables::*, shader::*};
use wgpu::{util::DeviceExt, Color};

mod context;
pub(crate) mod helpers;
mod renderables;
mod shader;

pub(crate) type InstanceType = [[f32; 4]; 4];
pub(crate) const INSTANCE_SIZE: usize = std::mem::size_of::<InstanceType>();

fn instance_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

impl Transform {
    pub(crate) fn as_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            glam::vec3(self.scale[0], self.scale[1], 1.0),
            glam::Quat::from_rotation_z(self.rotation),
            glam::vec3(
                self.position[0] * 2.0 - 1.0,
                self.position[1] * 2.0 - 1.0,
                0.0,
            ),
        )
    }
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
    pipeline_data: PipelineData,
    instance_count: u32,
    instance_data: DrawData,
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

    pub(crate) fn push_data(
        &mut self,
        pipeline_data: PipelineData,
        instance_count: u32,
        transform: Transform,
        default_pipeline_id: usize,
    ) {
        self.queue.push(RenderData {
            pipeline_data,
            instance_count,
            instance_data: DrawData {
                pipeline_id: self
                    .shader_queue
                    .last()
                    .copied()
                    .unwrap_or(default_pipeline_id),
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
