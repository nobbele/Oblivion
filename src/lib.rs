#![warn(clippy::clone_on_ref_ptr)]

pub(crate) use crate::internal::*;
pub use crate::{context::*, error::*, renderables::*, shader::*};

mod context;
mod error;
pub(crate) mod helpers;
mod internal;
mod renderables;
mod shader;

/// Vertex data.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    /// Position.
    pub position: mint::Point2<f32>,
    /// Color.
    pub color: rgb::RGBA<f32>,
    /// Texture coordinate.
    pub uv: mint::Point2<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

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
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const QUAD_VERTICES: &[Vertex] = &[
    // Top Left
    Vertex {
        position: mint::Point2 { x: 0.0, y: 0.0 },
        uv: mint::Point2 { x: 0.0, y: 0.0 },
        color: rgb::RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    },
    // Top Right
    Vertex {
        position: mint::Point2 { x: 1.0, y: 0.0 },
        uv: mint::Point2 { x: 1.0, y: 0.0 },
        color: rgb::RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    },
    // Bottom Left
    Vertex {
        position: mint::Point2 { x: 0.0, y: 1.0 },
        uv: mint::Point2 { x: 0.0, y: 1.0 },
        color: rgb::RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    },
    // Bottom Right
    Vertex {
        position: mint::Point2 { x: 1.0, y: 1.0 },
        uv: mint::Point2 { x: 1.0, y: 1.0 },
        color: rgb::RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
    },
];

pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 1, 3, 2];

// TODO make this a separate crate
/// Angle, uses radians internally.
///
/// Example usage:
/// ```rust
/// Angle::from_radians(std::f32::consts::FRAC_PI_2)
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    radians: f32,
}

impl Angle {
    /// Create angle from radians. (1 turn = 2π)
    #[inline]
    pub const fn from_radians(rad: f32) -> Self {
        Angle { radians: rad }
    }

    /// Create angle from degrees. (1 turn = 360°)
    #[inline]
    pub fn from_degrees(deg: f32) -> Self {
        Angle {
            radians: deg.to_radians(),
        }
    }

    /// Get the radian value of this angle. (1 turn = 2π)
    pub fn rad(self) -> f32 {
        self.radians
    }

    /// Get the degree value of this angle. (1 turn = 360°)
    pub fn deg(self) -> f32 {
        self.radians.to_degrees()
    }

    /// Sine function.
    #[inline]
    pub fn sin(self) -> f32 {
        self.radians.sin()
    }

    /// Cosine function.
    #[inline]
    pub fn cos(self) -> f32 {
        self.radians.cos()
    }
}

impl std::ops::Mul<f32> for Angle {
    type Output = Angle;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.radians *= rhs;
        self
    }
}

impl std::ops::Add<f32> for Angle {
    type Output = Angle;

    fn add(mut self, rhs: f32) -> Self::Output {
        self.radians += rhs;
        self
    }
}

/// Used to manipulate how an object is rendered.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Position or translation.
    pub position: mint::Point2<f32>,
    /// Scale.
    pub scale: mint::Vector2<f32>,
    /// Rotation.
    pub rotation: Angle,
    /// Offset.
    pub offset: mint::Point2<f32>,
}

impl Transform {
    pub(crate) fn as_matrix(&self, obj_dim: mint::Vector2<f32>) -> glam::Mat4 {
        let translate =
            glam::Mat4::from_translation(glam::vec3(self.position.x, self.position.y, 0.0));
        let offset_inv = glam::Mat4::from_translation(glam::vec3(
            -self.offset.x * obj_dim.x,
            -self.offset.y * obj_dim.y,
            0.0,
        ));
        let rotation = glam::Mat4::from_rotation_z(self.rotation.rad());
        let scale = glam::Mat4::from_scale(glam::vec3(self.scale.x, self.scale.y, 1.0));
        translate * rotation * scale * offset_inv
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0].into(),
            scale: [1.0, 1.0].into(),
            rotation: Angle::from_radians(0.0),
            // TODO maybe this shouldn't default to 0.5..
            offset: [0.5, 0.5].into(),
        }
    }
}

/// Stores a record and information about draw calls that can then be submitted to the context.
pub struct Render {
    shader_stack: Vec<usize>,
    render_groups: Vec<RenderGroup>,
    render_stack: Vec<usize>,
    // TODO make this a big buffer for all shader datas rather than just the active one?
    // TODO it would save lots of heap allocations.
    active_shader_data: Vec<u8>,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            shader_stack: Default::default(),
            render_groups: vec![RenderGroup::default()],
            render_stack: Default::default(),
            active_shader_data: Vec::new(),
        }
    }
}

impl Render {
    /// Creates a new render object.
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
        let uniform_extra = self.active_shader_data.clone();
        self.current_render_group().queue.push(RenderData {
            pipeline_data,
            instance_count,
            instance_data: DrawData {
                pipeline_id,
                transform,
                uniform_extra,
            },
        })
    }
}

/// Clears the screen with a color.
pub fn clear(render: &mut Render, color: impl Into<rgb::RGBA<f32, f32>>) {
    render.current_render_group().clear_color = Some(color.into());
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

pub fn set_shader_data<T: bytemuck::Pod>(render: &mut Render, data: &T) {
    render.active_shader_data = bytemuck::bytes_of(data).to_vec();
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
