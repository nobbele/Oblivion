use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, MeshBuffer, PipelineData, Render, Transform, Vertex};

/// Provides a way to build `Mesh` with convient functions such as triangle and rectangle generators.
#[derive(Default)]
pub struct MeshBuilder {
    vertex: Vec<Vertex>,
    base_index: u16,
    index: Vec<u16>,
}

// TODO correct UV
impl MeshBuilder {
    /// Creates a new mesh builder object.
    pub fn new() -> Self {
        MeshBuilder::default()
    }

    /// Adds a triangle to the builder.
    pub fn tri(
        &mut self,
        position: impl Into<mint::Point2<f32>>,
        size: impl Into<mint::Vector2<f32>>,
        color: impl Into<rgb::RGB<f32>>,
    ) -> &mut Self {
        let position = position.into();
        let size = size.into();
        let color = color.into();
        self.vertex.extend([
            Vertex {
                position: [position.x, position.y + size.y / 2.0].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
            Vertex {
                position: [position.x - size.x / 2.0, position.y - size.y / 2.0].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
            Vertex {
                position: [position.x + size.x / 2.0, position.y - size.y / 2.0].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
        ]);
        self.index
            .extend([self.base_index, self.base_index + 2, self.base_index + 1]);
        self.base_index += 3;
        self
    }

    /// Adds a quadrilateral to the builder.
    pub fn quad(
        &mut self,
        position: impl Into<mint::Point2<f32>>,
        size: impl Into<mint::Vector2<f32>>,
        color: impl Into<rgb::RGB<f32>>,
    ) -> &mut Self {
        let position = position.into();
        let size = size.into();
        let color = color.into();
        self.vertex.extend([
            Vertex {
                position: [position.x, position.y].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
            Vertex {
                position: [position.x + size.x, position.y].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
            Vertex {
                position: [position.x, position.y + size.y].into(),
                color,
                uv: [0.0, 0.0].into(),
            },
            Vertex {
                position: [position.x + size.x, position.y + size.y].into(),
                color,
                uv: [0.0, 0.0].into(),
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

    /// Builds the mesh object.
    pub fn build(&self, ctx: &GraphicsContext) -> Mesh {
        Mesh::new(ctx, &self.vertex, &self.index)
    }
}

/// Renderable mesh object. Essentially a list of shapes.
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
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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

        Mesh {
            data: PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group: Rc::new(bind_group),
                instance_buffer: Rc::clone(&ctx.identity_instance_buffer),
            },
        }
    }

    pub fn draw(&self, render: &mut Render, transform: Transform) {
        render.push_data(self.data.clone(), 1, transform, 0);
    }
}
