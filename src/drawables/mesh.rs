use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, MeshBuffer, OblivionResult, PipelineData, Render, Transform, Vertex};

#[derive(Copy, Clone, PartialEq, Debug)]
struct VertexBuilder {
    color: rgb::RGBA<f32>,
}

impl lyon::tessellation::StrokeVertexConstructor<Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> Vertex {
        let position = vertex.position();
        Vertex {
            position: [position.x, position.y].into(),
            uv: [0.0, 0.0].into(),
            color: self.color,
        }
    }
}

impl lyon::tessellation::FillVertexConstructor<Vertex> for VertexBuilder {
    fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> Vertex {
        let position = vertex.position();
        Vertex {
            position: [position.x, position.y].into(),
            uv: [0.0, 0.0].into(),
            color: self.color,
        }
    }
}

pub enum DrawMode {
    Fill { tolerance: f32 },
    Stroke { width: f32, tolerance: f32 },
}

impl DrawMode {
    pub fn fill() -> DrawMode {
        DrawMode::Fill {
            tolerance: lyon::tessellation::FillOptions::DEFAULT_TOLERANCE / 1000.0,
        }
    }

    pub fn stroke(width: f32) -> DrawMode {
        DrawMode::Stroke {
            width,
            tolerance: lyon::tessellation::FillOptions::DEFAULT_TOLERANCE / 1000.0,
        }
    }
}

impl Default for DrawMode {
    fn default() -> Self {
        Self::fill()
    }
}

/// Provides a way to build `Mesh` with convient functions such as triangle and rectangle generators.
#[derive(Clone)]
pub struct MeshBuilder {
    buffers: lyon::tessellation::VertexBuffers<Vertex, u16>,
}

// TODO correct UV
// TODO take an offset
impl MeshBuilder {
    /// Creates a new mesh builder object.
    pub fn new() -> Self {
        MeshBuilder {
            buffers: lyon::tessellation::VertexBuffers::new(),
        }
    }

    /// Adds a isosceles triangle to the builder.
    pub fn tri(
        &mut self,
        position: impl Into<mint::Point2<f32>>,
        size: impl Into<mint::Vector2<f32>>,
        color: impl Into<rgb::RGBA<f32>>,
        mode: DrawMode,
    ) -> OblivionResult<&mut Self> {
        let position = position.into();
        let size = size.into();
        let color = color.into();
        let mut buffers = &mut self.buffers;
        let mut bb = lyon::tessellation::BuffersBuilder::new(&mut buffers, VertexBuilder { color });
        match mode {
            DrawMode::Fill { tolerance } => {
                let mut tessellator = lyon::tessellation::FillTessellator::new();
                tessellator
                    .tessellate_polygon(
                        lyon::path::Polygon {
                            points: &[
                                lyon::math::point(position.x + size.x / 2.0, position.y),
                                lyon::math::point(position.x, position.y + size.y),
                                lyon::math::point(position.x + size.x, position.y + size.y),
                            ],
                            closed: true,
                        },
                        &lyon::tessellation::FillOptions::default().with_tolerance(tolerance),
                        &mut bb,
                    )
                    .unwrap();
            }
            DrawMode::Stroke { width, tolerance } => {
                let mut tessellator = lyon::tessellation::StrokeTessellator::new();
                tessellator
                    .tessellate_polygon(
                        lyon::path::Polygon {
                            points: &[
                                lyon::math::point(
                                    position.x + size.x / 2.0,
                                    position.y + (width.powi(2) + width.powi(2)).sqrt(),
                                ),
                                lyon::math::point(
                                    position.x + width / 2.0,
                                    position.y + size.y - width / 2.0,
                                ),
                                lyon::math::point(
                                    position.x + size.x - width / 2.0,
                                    position.y + size.y - width / 2.0,
                                ),
                            ],
                            closed: true,
                        },
                        &lyon::tessellation::StrokeOptions::default()
                            .with_tolerance(tolerance)
                            .with_line_width(width),
                        &mut bb,
                    )
                    .unwrap();
            }
        }
        Ok(self)
    }

    /// Adds a quadrilateral to the builder.
    pub fn quad(
        &mut self,
        position: impl Into<mint::Point2<f32>>,
        size: impl Into<mint::Vector2<f32>>,
        color: impl Into<rgb::RGBA<f32>>,
        mode: DrawMode,
    ) -> OblivionResult<&mut Self> {
        let position = position.into();
        let size = size.into();
        let color = color.into();
        let mut buffers = &mut self.buffers;
        let mut bb = lyon::tessellation::BuffersBuilder::new(&mut buffers, VertexBuilder { color });
        match mode {
            DrawMode::Fill { tolerance } => {
                let mut tessellator = lyon::tessellation::FillTessellator::new();
                tessellator
                    .tessellate_rectangle(
                        &lyon::math::rect(position.x, position.y, size.x, size.y),
                        &lyon::tessellation::FillOptions::default().with_tolerance(tolerance),
                        &mut bb,
                    )
                    .unwrap();
            }
            DrawMode::Stroke { width, tolerance } => {
                let mut tessellator = lyon::tessellation::StrokeTessellator::new();
                tessellator
                    .tessellate_rectangle(
                        &lyon::math::rect(
                            position.x + width / 2.0,
                            position.y + width / 2.0,
                            size.x - width,
                            size.y - width,
                        ),
                        &lyon::tessellation::StrokeOptions::default()
                            .with_tolerance(tolerance)
                            .with_line_width(width),
                        &mut bb,
                    )
                    .unwrap();
            }
        }
        Ok(self)
    }

    /// Adds a circle/ellipse to the builder.
    pub fn circle(
        &mut self,
        position: impl Into<mint::Point2<f32>>,
        size: impl Into<mint::Vector2<f32>>,
        color: impl Into<rgb::RGBA<f32>>,
        mode: DrawMode,
    ) -> OblivionResult<&mut Self> {
        let position = position.into();
        let size = size.into();
        let color = color.into();
        let mut buffers = &mut self.buffers;
        let mut bb = lyon::tessellation::BuffersBuilder::new(&mut buffers, VertexBuilder { color });
        match mode {
            DrawMode::Fill { tolerance } => {
                let mut tessellator = lyon::tessellation::FillTessellator::new();
                tessellator
                    .tessellate_ellipse(
                        lyon::math::point(position.x + size.x / 2.0, position.y + size.y / 2.0),
                        lyon::math::vector(size.x, size.y),
                        lyon::math::Angle::default(),
                        lyon::path::Winding::Positive,
                        &lyon::tessellation::FillOptions::default().with_tolerance(tolerance),
                        &mut bb,
                    )
                    .unwrap();
            }
            DrawMode::Stroke { width, tolerance } => {
                let mut tessellator = lyon::tessellation::StrokeTessellator::new();
                tessellator
                    .tessellate_ellipse(
                        lyon::math::point(position.x + size.x / 2.0, position.y + size.y / 2.0),
                        lyon::math::vector(size.x / 2.0 - width / 2.0, size.y / 2.0 - width / 2.0),
                        lyon::math::Angle::default(),
                        lyon::path::Winding::Positive,
                        &lyon::tessellation::StrokeOptions::default()
                            .with_tolerance(tolerance)
                            .with_line_width(width),
                        &mut bb,
                    )
                    .unwrap();
            }
        }
        Ok(self)
    }

    /// Builds the mesh object.
    pub fn build(&self, ctx: &GraphicsContext) -> Mesh {
        Mesh::new(ctx, &self.buffers.vertices, &self.buffers.indices)
    }
}

/// Renderable mesh object. Essentially a list of shapes.
#[derive(Clone)]
pub struct Mesh {
    data: PipelineData,
}

impl Mesh {
    /// Creates a new mesh object.
    pub fn new(ctx: &GraphicsContext, vertex: &[Vertex], index: &[u16]) -> Self {
        let mesh_buffer = MeshBuffer::from_slices(&ctx.device, vertex, index);

        // TODO move into context
        let texture = ctx.device.create_texture_with_data(
            &ctx.queue,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                label: Some("Oblivion_MeshTexture"),
            },
            &[255, 255, 255, 255],
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ctx.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Oblivion_MeshTextureBindGroup"),
        });

        let min_point = vertex.iter().fold(
            mint::Point2 {
                x: f32::MAX,
                y: f32::MAX,
            },
            |acc, v| mint::Point2 {
                x: acc.x.min(v.position.x),
                y: acc.y.min(v.position.y),
            },
        );
        let max_point = vertex.iter().fold(
            mint::Point2 {
                x: f32::MIN,
                y: f32::MIN,
            },
            |acc, v| mint::Point2 {
                x: acc.x.max(v.position.x),
                y: acc.y.max(v.position.y),
            },
        );

        let object_dimensions = mint::Vector2 {
            x: (max_point.x - min_point.x),
            y: (max_point.y - min_point.y),
        };

        Mesh {
            data: PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
                object_dimensions,
            },
        }
    }

    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.data.clone(), 1, transform, 0);
    }
}
