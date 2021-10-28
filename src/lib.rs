#![warn(clippy::clone_on_ref_ptr)]

pub(crate) use crate::internal::*;
pub use crate::{canvas::*, context::*, renderables::*, shader::*};

mod canvas;
mod context;
pub(crate) mod helpers;
mod internal;
mod renderables;
mod shader;

/// Vertex data.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

/// Used to manipulate how an object is rendered.
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
            // This is supposed to be p*2-1 for snorm but for reasons the -1 has to be in the shader.
            glam::vec3(self.position[0] * 2.0, self.position[1] * 2.0, 0.0),
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

/// Stores a record and information about draw calls that can then be submitted to the context.
pub struct Render {
    shader_stack: Vec<usize>,
    render_groups: Vec<RenderGroup>,
    render_stack: Vec<usize>,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            shader_stack: Default::default(),
            render_groups: vec![RenderGroup::default()],
            render_stack: Default::default(),
        }
    }
}

impl Render {
    pub fn new() -> Self {
        Render::default()
    }

    pub(crate) fn current_render_group(&mut self) -> &mut RenderGroup {
        &mut self.render_groups[self.render_stack.last().copied().unwrap_or(0)]
    }

    pub(crate) fn push_data(
        &mut self,
        pipeline_data: PipelineData,
        instance_count: u32,
        transform: Transform,
        default_pipeline_id: usize,
    ) {
        let pipeline_id = self
            .shader_stack
            .last()
            .copied()
            .unwrap_or(default_pipeline_id);
        self.current_render_group().queue.push(RenderData {
            pipeline_data,
            instance_count,
            instance_data: DrawData {
                pipeline_id,
                transform,
            },
        })
    }
}

/// Clears the screen with a color.
pub fn clear(render: &mut Render, color: wgpu::Color) {
    render.current_render_group().clear_color = Some(color);
    // We also need to clear the draw queue to give the illusion of the clear color overwriting everything else
    // This should produce the same behavior as just overwriting everything else though.
    render.current_render_group().queue.clear();
}

/// Sets an active shader. Use `oblivion::pop_shader` to unset it.
pub fn push_shader(render: &mut Render, shader: &Shader) {
    render.shader_stack.push(shader.pipeline_id);
}

/// Removes the active shader and goes back to the previous one.
pub fn pop_shader(render: &mut Render) {
    render.shader_stack.pop();
}

/// Sets an active canvas. Use `oblivion::pop_canvas` to unset it.
pub fn push_canvas(render: &mut Render, canvas: &Canvas) {
    render.render_groups.push(RenderGroup {
        target_id: TargetId::CanvasId(canvas.canvas_id),
        ..Default::default()
    });
    render.render_stack.push(render.render_groups.len() - 1);
}

/// Removes the active canvas and goes back to the previous one.
pub fn pop_canvas(render: &mut Render) {
    render.render_stack.pop();
}
