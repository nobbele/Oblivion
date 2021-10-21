use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, InstanceData, MeshBuffer, PipelineData, Render, RenderData, Vertex};

#[derive(Default)]
pub struct MeshBuilder {
    vertex: Vec<Vertex>,
    base_index: u16,
    index: Vec<u16>,
}

// TODO correct UV
impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder::default()
    }

    #[must_use]
    pub fn tri(mut self, position: [f32; 2], size: [f32; 2], color: [f32; 3]) -> Self {
        self.vertex.extend([
            Vertex {
                position: [position[0], position[1] + size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [position[0] - size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [position[0] + size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
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
            Vertex {
                position: [position[0] - size[0] / 2.0, position[1] + size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [position[0] + size[0] / 2.0, position[1] + size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [position[0] - size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
            },
            Vertex {
                position: [position[0] + size[0] / 2.0, position[1] - size[1] / 2.0],
                color,
                uv: [0.0, 0.0],
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
    //imp: Rc<MeshImpl>,
    data: Rc<PipelineData>,
}

impl Mesh {
    // TODO Result
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
                label: Some("Oblivion_Texture"),
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
            layout: &ctx.bind_group_layout,
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
            label: Some("Oblivion_TextureBindGroup"),
        });

        Mesh {
            data: Rc::new(PipelineData {
                mesh_buffer: Rc::new(mesh_buffer),
                bind_group,
            }),
        }
    }

    pub fn draw(&self, render: &mut Render) {
        render.queue.push(RenderData {
            pipeline_data: self.data.clone(),
            instance_data: InstanceData {
                pipeline_id: render.shader_queue.last().copied().unwrap_or(0),
            },
        })
    }
}
