use std::rc::Rc;

use crate::{
    pipelines::mesh_pipeline::MeshPipeline, vertices::mesh_vertex::MeshVertex, GraphicsContext,
    MeshBuffer, Render, Renderable,
};

#[derive(Default)]
pub struct MeshBuilder {
    vertex: Vec<MeshVertex>,
    base_index: u16,
    index: Vec<u16>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder::default()
    }

    #[must_use]
    pub fn tri(mut self, position: [f32; 2], size: [f32; 2], color: [f32; 3]) -> Self {
        self.vertex.extend([
            MeshVertex {
                position: [position[0], position[1] + size[1] / 2.0],
                color,
            },
            MeshVertex {
                position: [position[0] - size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
            },
            MeshVertex {
                position: [position[0] + size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
            },
        ]);
        self.index
            .extend([self.base_index, self.base_index + 2, self.base_index + 1]);
        self.base_index += 3;
        self
    }

    #[must_use]
    pub fn quad(mut self, position: [f32; 2], size: [f32; 2], color: [f32; 3]) -> Self {
        self.vertex.extend([
            MeshVertex {
                position: [position[0] - size[0] / 2.0, position[1] + size[1] / 2.0],
                color,
            },
            MeshVertex {
                position: [position[0] + size[0] / 2.0, position[1] + size[1] / 2.0],
                color,
            },
            MeshVertex {
                position: [position[0] - size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
            },
            MeshVertex {
                position: [position[0] + size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
            },
        ]);
        self.index.extend([
            self.base_index,
            self.base_index + 1,
            self.base_index + 2,
            self.base_index + 1,
            self.base_index + 3,
            self.base_index + 2,
        ]);
        self.base_index += 4;
        self
    }

    pub fn build(self, ctx: &GraphicsContext) -> Mesh {
        Mesh::new(ctx, &self.vertex, &self.index)
    }
}

pub struct Mesh {
    imp: Rc<MeshImpl>,
}

impl Mesh {
    // TODO Result
    pub fn new(ctx: &GraphicsContext, vertex: &[MeshVertex], index: &[u16]) -> Self {
        let mesh_buffer = MeshBuffer::from_slices(&ctx.device, vertex, index);
        Mesh {
            imp: Rc::new(MeshImpl {
                pipeline: Rc::clone(&ctx.mesh_pipeline),
                mesh_buffer,
            }),
        }
    }

    pub fn draw(&self, render: &mut Render) {
        render.queue.push(Rc::clone(&self.imp) as Rc<_>)
    }
}

pub struct MeshImpl {
    pipeline: Rc<MeshPipeline>,
    mesh_buffer: MeshBuffer,
}

impl Renderable for MeshImpl {
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline.render_pipeline);
        render_pass.set_vertex_buffer(0, self.mesh_buffer.vertex.0.slice(..));
        render_pass.set_index_buffer(
            self.mesh_buffer.index.0.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.mesh_buffer.index.1, 0, 0..1);
    }
}
