use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{GraphicsContext, InstanceData, PipelineData, Render, RenderData};

pub struct Image {
    //imp: Rc<ImageImpl>,
    data: Rc<PipelineData>,
}

impl Image {
    // TODO Result
    pub fn new(ctx: &GraphicsContext, width: u32, height: u32, data: &[u8]) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device.create_texture_with_data(
            &ctx.queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("Oblivion_Texture"),
            },
            data,
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

        Image {
            data: Rc::new(PipelineData {
                mesh_buffer: ctx.quad_mesh_buffer.clone(),
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
