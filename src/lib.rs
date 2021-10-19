#![warn(clippy::clone_on_ref_ptr)]
use std::rc::Rc;

pub use crate::{context::*, renderables::*};
use wgpu::{util::DeviceExt, Color};

mod context;
pub(crate) mod helpers;
pub(crate) mod pipelines;
mod renderables;
pub(crate) mod vertices;

pub trait Vertex: bytemuck::Pod {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub(crate) struct MeshBuffer {
    pub vertex: (wgpu::Buffer, u32),
    pub index: (wgpu::Buffer, u32),
}

impl MeshBuffer {
    pub fn from_slices<V: Vertex>(
        device: &wgpu::Device,
        vertex: &[V],
        index: &[u16],
    ) -> MeshBuffer {
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

pub(crate) trait Renderable {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}

#[derive(Default)]
pub struct Render {
    clear_color: Option<Color>,
    queue: Vec<Rc<dyn Renderable>>,
}

impl Render {
    pub fn new() -> Self {
        Render::default()
    }
}

pub fn clear(render: &mut Render, color: wgpu::Color) {
    render.clear_color = Some(color);
    // We also need to clear the draw queue to give the illusion of the clear color overwriting everything else
    // This should produce the same behavior as just overwriting everything else though.
    render.queue.clear();
}
